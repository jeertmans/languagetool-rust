use languagetool_rust::api::{check, server::ServerClient};
use std::borrow::Cow;

#[macro_export]
macro_rules! test_match_positions {
    ($name:ident, $text:expr, $(($x:expr, $y:expr)),*) => {
        #[tokio::test]
        async fn $name()  -> Result<(), Box<dyn std::error::Error>> {

            let client = ServerClient::from_env_or_default();
            let req = check::Request::default().with_text(Cow::Borrowed($text));
            let resp = client.check(&req).await.unwrap();
            let resp = check::ResponseWithContext::new(req.get_text().into_owned(), resp);

            let expected = vec![$(($x, $y)),*];
            let got = resp.iter_match_positions();

            assert_eq!(expected.len(), resp.response.matches.len());

            for ((lineno, lineof), got) in expected.iter().zip(got) {
                assert_eq!(*lineno, got.0);
                assert_eq!(*lineof, got.1);
            }

            Ok(())
        }
    };
}

test_match_positions!(
    test_match_positions_1,
    "Some phrase with a smal mistake.
i can drive a car",
    (1, 19),
    (2, 0)
);

test_match_positions!(
    test_match_positions_2,
    "Some phrase with
a smal mistake. i can
drive a car",
    (2, 2),
    (2, 16)
);

test_match_positions!(
    test_match_positions_3,
    "  Some phrase with a smal
mistake.

  i can drive a car",
    (1, 0),
    (1, 21),
    (4, 0),
    (4, 2)
);

test_match_positions!(
    test_match_positions_4,
    "Some phrase with a smal mistake.
i can drive a car
Some phrase with a smal mistake.",
    (1, 19),
    (2, 0),
    (3, 19)
);
