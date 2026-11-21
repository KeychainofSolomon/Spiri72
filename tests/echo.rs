use spiri72::agens::sine_echo;

#[test]
fn the_greeting_said_back_is_taken_off() {
    let q = "Hail, Furfur! What is the link between desire and death?";
    assert_eq!(
        sine_echo(q, "Hail, Furfur! The link is the scythe.".into()),
        "The link is the scythe."
    );
    assert_eq!(sine_echo(q, "hail, furfur!  The link.".into()), "The link.");
}

#[test]
fn only_an_exact_echo_is_taken_off() {
    let q = "Hail, Furfur! What is the link?";
    for a in [
        "Hail, magician! The link is the scythe.",
        "The link is the scythe. Hail, Furfur!",
        "Hail, Furfur!",
    ] {
        assert_eq!(sine_echo(q, a.into()), a);
    }
    assert_eq!(
        sine_echo("no stop at all", "no stop at all, said".into()),
        "no stop at all, said"
    );
}
