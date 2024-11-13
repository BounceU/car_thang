// Necessary to compile for Slint

fn main() {
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
}
