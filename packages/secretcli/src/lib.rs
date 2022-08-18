pub mod cli_types;
pub mod secretcli;

pub const LABEL_ALPHABET: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub fn generate_label(size: usize) -> String {
    nanoid::nanoid!(size, &LABEL_ALPHABET)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gen_label() {
        let length: usize = 20;
        assert_eq!(length, generate_label(length).capacity())
    }
}
