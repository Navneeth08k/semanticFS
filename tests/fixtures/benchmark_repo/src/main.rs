mod auth;
mod map_logic;

fn main() {
    let token = auth::issue_token("demo-user");
    let summary = map_logic::directory_purpose("src");
    println!("token={} summary={}", token, summary);
}
