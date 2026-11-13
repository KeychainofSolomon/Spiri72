use annulus::lingua::registra::goetia;
use spiri72::custos::examine;

fn spirit(n: i64) -> &'static annulus::lingua::registra::Spirit {
    goetia().by_number(n).unwrap()
}

#[test]
fn a_spirit_true_to_its_tables_passes() {
    // Bael: king, 66 legions.
    let bael = spirit(1);
    for text in [
        "I am Bael, a king of the East, and I rule 66 legions.",
        "I am Bael. I rule sixty-six legions of spirits.",
        "I am the great king Bael; sixty and six legions are mine to command.",
        "Kneel, mortal. I command sixty-six of my legions.",
        "My name is Bael.",
        "I make men go invisible.",
    ] {
        assert!(
            examine(bael, text).is_empty(),
            "{text}: {:?}",
            examine(bael, text)
        );
    }
}

#[test]
fn wrong_legions_are_found() {
    let bael = spirit(1);
    let f = examine(bael, "I am Bael, and I rule thirty legions.");
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].contains("30") && f[0].contains("66"), "{f:?}");
    assert_eq!(examine(bael, "I rule 200 legions").len(), 1);
    assert_eq!(
        examine(bael, "I lead two hundred and six legions.").len(),
        1
    );
}

#[test]
fn another_spirits_legions_are_not_its_own() {
    // Paimon rules 200; Bael saying so is no fault.
    let bael = spirit(1);
    assert!(examine(bael, "Paimon rules 200 legions, and I bow to none.").is_empty());
    assert!(examine(bael, "I envy Paimon's 200 legions.").is_empty());
}

#[test]
fn a_wrong_rank_is_found() {
    let bael = spirit(1);
    let f = examine(bael, "I am a duke of hell.");
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].contains("duke") && f[0].contains("king"), "{f:?}");
    assert_eq!(examine(bael, "I'm the mighty President").len(), 1);
    // Serving a king is not being one, and this is Bael's own rank anyway.
    assert!(examine(bael, "I am the king's own herald.").is_empty());
    assert!(examine(bael, "I am a king.").is_empty());
    assert!(examine(bael, "I am Rex.").is_empty());
}

#[test]
fn a_spirit_of_two_ranks_may_name_either() {
    // Vine is king and earl.
    let vine = spirit(45);
    assert!(examine(vine, "I am an earl.").is_empty());
    assert!(examine(vine, "I am a king.").is_empty());
    assert_eq!(examine(vine, "I am a duke.").len(), 1);
}

#[test]
fn calling_itself_another_spirit_is_found() {
    let bael = spirit(1);
    let f = examine(bael, "I am Paimon, and I teach all arts.");
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].contains("paimon") && f[0].contains("Bael"), "{f:?}");
    assert_eq!(examine(bael, "My name is Furfur.").len(), 1);
    assert!(examine(spirit(25), "I am Glasya-Labolas.").is_empty());
}

#[test]
fn deceit_about_the_world_is_left_to_the_angel() {
    let bael = spirit(1);
    assert!(examine(
        bael,
        "The gold lies under the third stone. You have nine days."
    )
    .is_empty());
}
