/// # Errors
#[allow(clippy::needless_pass_by_value)]
pub fn print_writer(
    stream: &mut impl std::io::Write,
    prompt: impl ToString,
) -> std::io::Result<()> {
    stream
        .write_all(prompt.to_string().as_bytes())
        .and_then(|()| stream.flush())
}
