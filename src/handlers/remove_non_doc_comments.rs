///
/// This is ugly AF, but it works so let this Frankenstein exist XD
///
pub fn remove_non_doc_comments(mut code: String) -> String {
	let clone = code.clone();
	let mut offset = 0;
	let mut iter = clone.chars().enumerate();
	while let Some(next) = iter.next() {
		if next.1 == ';' {
			let start = next.0;
			if matches!(iter.clone().next(), Some((_, '['))) {
				iter.next();
				let mut count = 1;
				while let Some(next) = iter.next() {
					if next.1 == ']' {
						if matches!(iter.clone().next(), Some((_, ';'))) {
							iter.next();
							count -= 1;
						}
					} else if next.1 == ';' {
						if matches!(iter.clone().next(), Some((_, '['))) {
							iter.next();
							count += 1;
						}
					}

					if count == 0 {
						break
					}
				}
				if count != 0 {
					panic!("unterminated multiline comment")
				}
			} else {
				while let Some(char) = iter.next() {
					if char.1 == '\n' {
						break
					}
				}
			}
			code = code[..start - offset].to_string() + if let Some(end) = iter.clone().next() {
				let oldoffset = offset;
				offset += end.0 - start;
				&code[end.0 - oldoffset..]
			} else {
				""
			}
		}
	}
	code
}
