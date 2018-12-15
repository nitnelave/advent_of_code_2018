use std::collections::vec_deque::VecDeque;

type Recipe = u8;

/// Iterator that generates the infinite list of recipes.
#[derive(Debug, Clone)]
struct RecipeGenerator {
    recipes: Vec<Recipe>,
    elf_positions: [usize; 2],
    next_index: usize,
}

impl Default for RecipeGenerator {
    fn default() -> Self {
        Self {
            recipes: vec![3, 7],
            elf_positions: [0, 1],
            next_index: 0,
        }
    }
}

impl std::iter::Iterator for RecipeGenerator {
    type Item = Recipe;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index >= self.recipes.len() {
            // Compute the next recipe.
            self.generate_more_recipes();
            // Update the elves.
            self.move_elves();
        }
        assert!(self.next_index < self.recipes.len());
        self.next_index += 1;
        Some(self.recipes[self.next_index - 1])
    }
}

impl RecipeGenerator {
    fn generate_more_recipes(&mut self) {
        let sum: u8 = self.elf_positions.iter().map(|i| self.recipes[*i]).sum();
        if sum >= 10 {
            self.recipes.push(sum / 10);
        }
        self.recipes.push(sum % 10);
    }

    fn move_elves(&mut self) {
        for e in &mut self.elf_positions {
            // Modulo for wrapping.
            *e = (*e + self.recipes[*e] as usize + 1) % self.recipes.len();
        }
    }
}

fn generate_recipes() -> RecipeGenerator {
    RecipeGenerator::default()
}

/// Match a pattern against a stream of recipes.
struct RecipeMatcher {
    // The input pattern.
    pattern: Vec<Recipe>,
    // The last n recipes, with n being the size of the pattern.
    buffer: VecDeque<Recipe>,
}

impl RecipeMatcher {
    fn new(pattern: Vec<Recipe>, buffer: Vec<Recipe>) -> Self {
        assert_eq!(pattern.len(), buffer.len());
        Self {
            pattern,
            buffer: VecDeque::from(buffer),
        }
    }

    /// Returns whether the stream of recipes, with the `new_recipe` added, matches the pattern.
    fn matches(&mut self, new_recipe: Recipe) -> bool {
        // Cycle the recipes.
        self.buffer.pop_front();
        self.buffer.push_back(new_recipe);
        // Check if the buffer matches the pattern.
        self.pattern
            .iter()
            .zip(self.buffer.iter())
            .all(|(a, b)| *a == *b)
    }
}

/// Convert a string of digits to a vector of u8.
fn str_to_vec(s: &str) -> Vec<u8> {
    s.bytes().map(|c| c - b'0').collect()
}

/// Returns the list of 10 recipes after `steps` step.
pub fn find_score_after_steps(steps: usize) -> Vec<u8> {
    // Skip the first `steps`, then take the next 10.
    generate_recipes().skip(steps).take(10).collect()
}

/// Returns the number of recipes to the left of the first time that the `input` pattern appears.
pub fn find_first_pattern(input: &str) -> usize {
    let pattern = str_to_vec(input);
    let pat_len = pattern.len();
    let recipe_generator = generate_recipes();
    let mut recipe_matcher =
        RecipeMatcher::new(pattern, recipe_generator.clone().take(pat_len).collect());
    recipe_generator
        .enumerate()
        .skip(pat_len)
        .skip_while(|(_, r)| !recipe_matcher.matches(*r))
        .next()
        .unwrap()
        .0
        - pat_len
        + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_score_after_steps_test() {
        assert_eq!(find_score_after_steps(5), str_to_vec("0124515891"));
        assert_eq!(find_score_after_steps(9), str_to_vec("5158916779"));
        assert_eq!(find_score_after_steps(18), str_to_vec("9251071085"));
        assert_eq!(find_score_after_steps(2018), str_to_vec("5941429882"));
    }

    #[test]
    fn find_first_pattern_test() {
        assert_eq!(find_first_pattern("51589"), 9);
        assert_eq!(find_first_pattern("01245"), 5);
        assert_eq!(find_first_pattern("92510"), 18);
        assert_eq!(find_first_pattern("59414"), 2018);
    }
}
