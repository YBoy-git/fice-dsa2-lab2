use std::{
    env, fs,
    io::{BufRead, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    if args.len() != 4 {
        panic!(
            "Incorrect argument set
    please, pass <PATH_TO_INPUT_FILE> <ID_OF_USER_TO_COMPARE> <PATH_TO_OUTPUT_FILE>"
        );
    }

    let _ = args.next();
    let input_file = PathBuf::from_str(&args.next().unwrap())?;
    if !input_file.exists() {
        panic!("Input file does not exist");
    }
    let target_user_id = args.next().unwrap().parse::<u32>()?;
    let output_file = PathBuf::from_str(&args.next().unwrap())?;

    make_recommendation_rating(&input_file, target_user_id, &output_file)?;

    Ok(())
}

fn make_recommendation_rating(
    input_file: &Path,
    target_user_id: u32,
    output_file: &Path,
) -> anyhow::Result<()> {
    let ratings = parse_input_file(input_file)?;

    let mut collisions = get_rating_collisions(&ratings, target_user_id);

    let mut inversions = collisions
        .iter_mut()
        .map(|(id, collision)| (*id, sort_and_count_inversions(collision)))
        .collect::<Vec<_>>();

    inversions.sort_by(|a, b| a.1.cmp(&b.1));

    let mut content_to_write = Vec::new();

    writeln!(content_to_write, "{target_user_id}")?;
    for (id, inversion) in &inversions {
        writeln!(content_to_write, "{id} {inversion}")?;
    }

    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_file, &content_to_write)?;

    Ok(())
}

fn parse_input_file(input_file: &Path) -> anyhow::Result<Vec<(u32, Vec<u32>)>> {
    let input = fs::read(input_file)?;
    let mut input_lines = input.lines();

    let info_line = input_lines.next().unwrap()?;
    let mut info_line_split = info_line.split(' ');

    let _users = info_line_split.next().unwrap().parse::<u32>()?;
    let _films = info_line_split.next().unwrap().parse::<u32>()?;

    let mut ratings = Vec::new();

    for user_line in input_lines {
        let user_line = user_line?;
        let mut user_line_split = user_line.split(' ');
        let user_id = user_line_split.next().unwrap().parse::<u32>()?;
        let user_rating = user_line_split
            .map(|mark| mark.parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        ratings.push((user_id, user_rating));
    }

    Ok(ratings)
}

fn get_rating_collisions(ratings: &[(u32, Vec<u32>)], target_user_id: u32) -> Vec<(u32, Vec<u32>)> {
    let users = ratings.len();
    let films = ratings.first().unwrap().1.len();

    let target_user_rating = &ratings
        .iter()
        .filter(|(id, _)| *id == target_user_id)
        .collect::<Vec<_>>()
        .pop()
        .unwrap()
        .1;

    let mut collisions = Vec::with_capacity(users - 1);

    for (id, rating) in ratings.iter().filter(|(id, _)| *id != target_user_id) {
        let mut collision = Vec::with_capacity(films);
        for i in 1..=films {
            let index = target_user_rating
                .iter()
                .position(|mark| *mark == i as u32)
                .unwrap();
            collision.push(rating[index]);
        }
        collisions.push((*id, collision));
    }

    collisions
}

fn sort_and_count_inversions(array: &mut [u32]) -> u32 {
    let length = array.len();
    let mid = length / 2;

    if length == 1 {
        return 0;
    }

    let left_inversions = sort_and_count_inversions(&mut array[0..mid]);
    let right_inversions = sort_and_count_inversions(&mut array[mid..]);
    let split_inversions = merge_and_count_split_inversions(array, mid);

    left_inversions + right_inversions + split_inversions
}

fn merge_and_count_split_inversions(array: &mut [u32], mid: usize) -> u32 {
    let length = array.len();
    let l_length = mid;

    let mut split_intersections = 0;

    let mut l_pointer = 0;
    let mut r_pointer = mid;

    let mut merged = Vec::with_capacity(length);

    while l_pointer < mid || r_pointer < length {
        if l_pointer >= mid {
            merged.push(array[r_pointer]);
            r_pointer += 1;
            continue;
        }
        if r_pointer >= length {
            merged.push(array[l_pointer]);
            l_pointer += 1;
            continue;
        }

        if array[l_pointer] <= array[r_pointer] {
            merged.push(array[l_pointer]);
            l_pointer += 1;
        } else {
            merged.push(array[r_pointer]);
            r_pointer += 1;
            split_intersections += l_length - l_pointer;
        }
    }

    array[..].copy_from_slice(&merged[..]);

    split_intersections as u32
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        str::FromStr,
    };

    use crate::make_recommendation_rating;

    fn compare_output(expected: &Path, actual: &Path) -> anyhow::Result<bool> {
        let expected = fs::read(expected)?;
        let actual = fs::read(actual)?;

        Ok(expected == actual)
    }

    #[test]
    fn test_data() -> anyhow::Result<()> {
        let input_dir = Path::new("data/input");
        let expected_output_dir = Path::new("data/output/expected");
        let actual_output_dir = Path::new("data/output/actual");

        fs::create_dir_all(input_dir)?;
        fs::create_dir_all(expected_output_dir)?;
        fs::create_dir_all(actual_output_dir)?;

        for input_file in input_dir.read_dir()? {
            let input_file = input_file?;
            if input_file.path().extension().unwrap() != "txt" {
                eprintln!(
                    "{} not a .txt file, skipping...",
                    input_file.path().display()
                );
                continue;
            }

            let dir_with_test_cases = PathBuf::from_str(&format!(
                "{}/{}",
                expected_output_dir.to_str().unwrap(),
                input_file.path().file_stem().unwrap().to_str().unwrap()
            ))?;
            if !dir_with_test_cases.exists() {
                eprintln!(
                    "No tests found for {}, skipping",
                    input_file.path().display()
                );
                continue;
            }

            for test_case_file in dir_with_test_cases.read_dir()? {
                let test_case_file = test_case_file?;

                let target_user_id = test_case_file
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .parse::<u32>()?;

                let output_file_path = PathBuf::from_str(&format!(
                    "{}/{}/{}.txt",
                    actual_output_dir.display(),
                    input_file.path().file_stem().unwrap().to_str().unwrap(),
                    target_user_id
                ))?;

                if let Some(parent) = output_file_path.parent() {
                    println!("parent: {}", parent.display());
                    fs::create_dir_all(parent)?;
                }

                make_recommendation_rating(&input_file.path(), target_user_id, &output_file_path)?;

                assert!(
                    compare_output(&test_case_file.path(), &output_file_path).unwrap(),
                    "
Files are not identical.
    input_file: {}
    expected_output_file: {},
    actual_output_file: {}
",
                    input_file.path().display(),
                    test_case_file.path().display(),
                    output_file_path.display()
                );
            }
        }

        Ok(())
    }
}
