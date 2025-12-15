fn leaf(n: i32) -> i32 {
    n + 1
}

fn mid(n: i32) -> i32 {
    let a = leaf(n);
    let b = leaf(a);
    a + b
}

fn top() -> i32 {
    mid(10)
}

fn main() {
    let v = top();
    println!("smoke: {v}");
}
