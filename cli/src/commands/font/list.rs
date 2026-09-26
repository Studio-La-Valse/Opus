use crate::smufl_fonts::{SmuflRoots, discover};

pub fn run() {
    let roots = SmuflRoots::for_this_system();
    let fonts = discover(&roots);

    if fonts.is_empty() {
        println!("No SMuFL fonts found. Searched:");
        for root in roots.all() {
            println!("  {}", root.join("SMuFL").join("Fonts").display());
        }
        println!("Install one with `opus font install <folder>`.");
        return;
    }

    for font in fonts {
        println!("{}\t{}", font.name, font.metadata.display());
    }
}
