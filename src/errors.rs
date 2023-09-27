pub enum Maybe<T> {
    Ok(T),
    Err(String),
    Not,
}

impl<T> Maybe<T> {
    pub fn expect(self, msg: &str) -> T {
        match self {
            Maybe::Ok(t) => t,
            _ => {
                if msg == "" {
                    panic!("expected Ok")
                } else {
                    panic!("expected: {}", msg)
                }
            }
        }
    }

    pub fn expect_err(self, msg: &str) -> String {
        match self {
            Maybe::Err(msg) => msg,
            _ => {
                if msg == "" {
                    panic!("expected Err")
                } else {
                    panic!("expected: {}", msg)
                }
            }
        }
    }

    pub fn expect_not(self, msg: &str) {
        match self {
            Maybe::Not => (),
            _ => {
                if msg == "" {
                    panic!("expected Not")
                } else {
                    panic!("expected: {}", msg)
                }
            }
        }
    }
}
