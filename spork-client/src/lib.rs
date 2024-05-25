pub fn client(left: usize, right: usize) -> usize {
    left + right + 10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = client(2, 2);
        assert_eq!(result, 4);
    }
}
