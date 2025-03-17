mod puzzle;

use puzzle::Puzzle;



fn main() {
    let mut puzzle = Puzzle::new(4);

    println!("Initial Puzzle:\n{}", puzzle);

    puzzle.shuffle();
    let mut original = puzzle.clone();

    println!("Shuffled Puzzle:\n{}", puzzle);

    let output = puzzle.solve().unwrap();
    println!("Found optimal solution in with: {} moves", output.len());

    for item in output {
        original.apply_move(item);
        println!("{}\n{}\n", item, original)
    }
}
