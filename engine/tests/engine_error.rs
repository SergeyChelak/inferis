use std::error::Error;

use engine::{ComponentStorage, EngineError};

#[test]
fn display_reads_as_a_sentence_not_as_debug() {
    let err = EngineError::unexpected_state("maze component not found");
    assert_eq!(
        err.to_string(),
        "unexpected state: maze component not found"
    );
    assert_ne!(format!("{err:?}"), err.to_string());
}

#[test]
fn every_variant_says_something() {
    let mut storage = ComponentStorage::new();
    let dead = storage.add_entity();
    storage.remove_entity(dead);

    let errors = [
        EngineError::ComponentNotRegistered,
        EngineError::ComponentAlreadyRegistered,
        EngineError::ComponentCountOverflow,
        EngineError::component_not_found("Velocity"),
        EngineError::EntityNotAlive(dead),
        EngineError::TextureNotFound("wall1".into()),
        EngineError::SceneNotFound,
        EngineError::FileAccessError("assets.bin".into()),
        EngineError::ResourceParseError("bad header".into()),
        EngineError::ResourceNotFound("sound_npc_pain".into()),
        EngineError::sdl("no free channels"),
        EngineError::MazeGenerationFailed("no empty spaces".into()),
        EngineError::unexpected_state("nothing to report"),
    ];
    for err in errors {
        let text = err.to_string();
        assert!(!text.is_empty(), "{err:?} has an empty Display");
        // prose, not the variant name the derived Debug would print
        assert!(text.contains(' '), "{err:?} renders as {text:?}");
        assert_ne!(
            text,
            format!("{err:?}"),
            "{err:?} has no Display of its own"
        );
    }
}

#[test]
fn interoperates_with_box_dyn_error() {
    fn fallible() -> Result<(), Box<dyn Error>> {
        // the `?` conversion the engine could not take part in before
        Err(EngineError::SceneNotFound)?;
        Ok(())
    }
    let err = fallible().unwrap_err();
    assert_eq!(err.to_string(), "scene not found");
    assert!(err.downcast_ref::<EngineError>().is_some());
}
