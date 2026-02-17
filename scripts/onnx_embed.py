#!/usr/bin/env python3
import argparse
import json
import sys

import numpy as np
import onnxruntime as ort
from transformers import AutoTokenizer


def l2_normalize(v: np.ndarray) -> np.ndarray:
    norm = np.linalg.norm(v)
    if norm == 0:
        return v
    return v / norm


def mean_pool(last_hidden_state: np.ndarray, attention_mask: np.ndarray) -> np.ndarray:
    mask = attention_mask.astype(np.float32)
    mask = np.expand_dims(mask, axis=-1)
    summed = np.sum(last_hidden_state * mask, axis=1)
    counts = np.clip(np.sum(mask, axis=1), 1e-9, None)
    return summed / counts


def build_feed(session, enc):
    input_names = {i.name for i in session.get_inputs()}
    feed = {}
    if "input_ids" in input_names:
        feed["input_ids"] = enc["input_ids"].astype(np.int64)
    if "attention_mask" in input_names:
        feed["attention_mask"] = enc["attention_mask"].astype(np.int64)
    if "token_type_ids" in input_names and "token_type_ids" in enc:
        feed["token_type_ids"] = enc["token_type_ids"].astype(np.int64)
    return feed

def postprocess_embeddings(outputs: np.ndarray, attention_mask: np.ndarray, dim: int):
    if outputs.ndim == 3:
        pooled = mean_pool(outputs, attention_mask)
    elif outputs.ndim == 2:
        pooled = outputs
    else:
        pooled = outputs.reshape(outputs.shape[0], -1)

    out = []
    for i in range(pooled.shape[0]):
        emb = pooled[i].astype(np.float32)
        emb = l2_normalize(emb)
        if emb.shape[0] > dim:
            emb = emb[:dim]
        elif emb.shape[0] < dim:
            emb = np.pad(emb, (0, dim - emb.shape[0]), mode="constant")
        out.append(emb)
    return out


def compute_embedding(session, tokenizer, text: str, max_length: int, dim: int) -> np.ndarray:
    enc = tokenizer(
        text,
        return_tensors="np",
        truncation=True,
        max_length=max_length,
        padding="max_length",
    )
    feed = build_feed(session, enc)
    outputs = session.run(None, feed)
    if len(outputs) == 0:
        raise RuntimeError("model produced no outputs")
    embs = postprocess_embeddings(outputs[0], enc["attention_mask"], dim)
    return embs[0]


def compute_embeddings(session, tokenizer, texts, max_length: int, dim: int):
    if not texts:
        return []
    enc = tokenizer(
        texts,
        return_tensors="np",
        truncation=True,
        max_length=max_length,
        padding=True,
    )
    feed = build_feed(session, enc)
    outputs = session.run(None, feed)
    if len(outputs) == 0:
        raise RuntimeError("model produced no outputs")
    return postprocess_embeddings(outputs[0], enc["attention_mask"], dim)


def print_error(msg: str):
    print(json.dumps({"error": msg}), flush=True)


def serve_loop(session, tokenizer, max_length: int, dim: int):
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            payload = json.loads(line)
            op = payload.get("op", "")
            if op == "ping":
                print(json.dumps({"ok": True, "backend": "onnxruntime"}), flush=True)
                continue

            texts = payload.get("texts")
            if texts is None:
                # Backward-compatible single-text request shape.
                text = payload.get("text", "")
                texts = [text]

            embs = compute_embeddings(session, tokenizer, texts, max_length, dim)
            print(json.dumps({"embeddings": [e.tolist() for e in embs]}), flush=True)
        except Exception as e:
            print_error(str(e))


def one_shot(session, tokenizer, text: str, max_length: int, dim: int):
    emb = compute_embedding(session, tokenizer, text, max_length, dim)
    print(json.dumps({"embeddings": [emb.tolist()]}), flush=True)


def main():
    parser = argparse.ArgumentParser(description="SemanticFS ONNX embedding sidecar")
    parser.add_argument("--model", required=True)
    parser.add_argument("--tokenizer", required=True)
    parser.add_argument("--dim", type=int, required=True)
    parser.add_argument("--max-length", type=int, default=512)
    parser.add_argument("--serve", action="store_true")
    parser.add_argument("--text", default="")
    parser.add_argument("--provider", default="CPUExecutionProvider")
    parser.add_argument("--intra-threads", type=int, default=0)
    parser.add_argument("--inter-threads", type=int, default=0)
    args = parser.parse_args()

    tokenizer = AutoTokenizer.from_pretrained(args.tokenizer, use_fast=True)
    sess_opts = ort.SessionOptions()
    if args.intra_threads and args.intra_threads > 0:
        sess_opts.intra_op_num_threads = args.intra_threads
    if args.inter_threads and args.inter_threads > 0:
        sess_opts.inter_op_num_threads = args.inter_threads
    session = ort.InferenceSession(args.model, sess_options=sess_opts, providers=[args.provider])

    if args.serve:
        serve_loop(session, tokenizer, args.max_length, args.dim)
    else:
        one_shot(session, tokenizer, args.text, args.max_length, args.dim)


if __name__ == "__main__":
    main()
