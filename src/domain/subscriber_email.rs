use validator::Validate;

#[derive(Debug)]
pub struct SubscriberEmail(String);

#[derive(Validate)]
struct Email {
    #[validate(email)]
    address: String,
}
impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        let res = Email { address: s.clone() };
        match res.validate() {
            Ok(_) => Ok(Self(s)),
            Err(_) => Err(String::from("invalid email")),
        }
    }
}
impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::{Arbitrary, Gen};

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        claim::assert_err!(SubscriberEmail::parse(email));
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        claim::assert_err!(SubscriberEmail::parse(email));
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        claim::assert_err!(SubscriberEmail::parse(email));
    }

    // #[test]
    // fn valid_emails_are_parsed_successfully() {
    //     let email = SafeEmail().fake();
    //     claim::assert_ok!(SubscriberEmail::parse(email));
    // }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let _ = g;
            let email = SafeEmail().fake();
            Self(email)
        }
        // fn arbitrary(g: &mut Gen) -> Self {
        //     let email = SafeEmail().fake_with_rng(g);
        //     Self(email)
        // }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
