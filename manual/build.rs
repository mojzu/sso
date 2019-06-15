use skeptic::*;

fn main() {
    let mdbook_files = markdown_files_of_directory("book-src/");
    generate_doc_tests(&mdbook_files);
}
