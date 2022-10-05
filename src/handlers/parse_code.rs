use crate::*;

pub fn parse_code(input: &mut ParseInput) -> Result <()> {
    loop {
        remove_newlines(input);

        if input.is_exhausted() {
            break
        }

        let item = Item::parse(input)?;

        input.top_level_items.push(item)
    }

    Result(Ok(()))
}
