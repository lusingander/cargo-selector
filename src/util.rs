pub fn digits(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut n = n;
    let mut c = 0;
    while n > 0 {
        n /= 10;
        c += 1;
    }
    c
}
