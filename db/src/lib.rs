pub fn read() {
    println!("db read");
}

pub fn write() {
    println!("db write");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
