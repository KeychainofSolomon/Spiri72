use std::path::PathBuf;

use spiri72::initium::{clean_path, read_ini, write_ini, Ini};

#[test]
fn a_dropped_path_is_unquoted_and_trimmed() {
    assert_eq!(
        clean_path("  \"C:\\llama\\llama-server.exe\"\r\n"),
        "C:\\llama\\llama-server.exe"
    );
    assert_eq!(clean_path("'/home/m/model.gguf' "), "/home/m/model.gguf");
    assert_eq!(clean_path("plain.gguf"), "plain.gguf");
}

#[test]
fn the_remembered_paths_survive_the_round_trip() {
    let ini = Ini {
        server: Some(PathBuf::from("C:\\llama\\llama-server.exe")),
        model: Some(PathBuf::from("D:\\models\\Qwen3-8B Q4_K_M.gguf")),
        eyes: Some(PathBuf::from("D:\\models\\Qwen2.5-VL-3B.gguf")),
        eyes_mmproj: Some(PathBuf::from("D:\\models\\mmproj-Qwen2.5-VL-3B.gguf")),
        blind: false,
        heartbeat: Some(45),
    };
    assert_eq!(read_ini(&write_ini(&ini)), ini);
    assert_eq!(read_ini("# nothing here\nnoise\n"), Ini::default());
    let blind = Ini {
        blind: true,
        ..Ini::default()
    };
    assert_eq!(read_ini(&write_ini(&blind)), blind);
}

#[test]
fn the_eyes_have_room_for_an_image_and_its_reading() {
    use spiri72::initium::{EYES_CONTEXT, EYES_IMAGE_TOKENS};
    // the longest reply the eyes are asked for is 1536 tokens (a page copied out);
    // 200 more for the request itself
    assert!(EYES_IMAGE_TOKENS + 1536 + 200 <= EYES_CONTEXT);
}
