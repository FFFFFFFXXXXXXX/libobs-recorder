fn main() {
    build_helper::build().unwrap();
    build_helper::copy_artifact_dependency().unwrap()
}
