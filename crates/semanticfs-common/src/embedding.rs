use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn embed_text_hash(text: &str, dim: usize) -> Vec<f32> {
    let dim = dim.max(1);
    let mut vec = vec![0.0f32; dim];

    for token in tokenize(text) {
        let idx = hash_to_index(&token, dim);
        vec[idx] += 1.0;
    }

    // L2 normalize for cosine similarity.
    let norm = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vec {
            *v /= norm;
        }
    }

    vec
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn tokenize(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
}

fn hash_to_index(token: &str, dim: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    (hasher.finish() as usize) % dim
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similar_text_has_higher_cosine() {
        let d = 64;
        let a = embed_text_hash("auth token validation", d);
        let b = embed_text_hash("token auth validation", d);
        let c = embed_text_hash("image processing pipeline", d);

        assert!(cosine_similarity(&a, &b) > cosine_similarity(&a, &c));
    }
}
