use std::path::PathBuf;
  
    pub fn cut_off_between_strings(start: &str, end: &str, include_ending: bool, line: &mut String, range: Option<&(usize, usize)>) -> bool {
        if let Some(s) = line.find(start) {
            if let Some((min, _)) = range { if s < *min {return false;}}
            if let Some(mut e) = line.find(end) {
                if include_ending { e += end.len() }
                if let Some((_, max)) = range { if e > *max {return false;}}
                line.replace_range(s..e, "");
                *line = line.trim().to_string();
                return true;
            }
            *line = "".to_string();
        }

        false
    }

    pub fn remove_comments(line: &mut String) -> bool{
        let mut is_in_quotes = false;
        let mut is_escaped = false;
        let mut starting_index = 0;
        let mut ranges_to_inspect = Vec::new();
        for (index, c) in line.char_indices() {
            if c == '"' && !is_escaped {
                is_in_quotes = !is_in_quotes; 
                if is_in_quotes {
                    ranges_to_inspect.push((starting_index, index));
                } else {
                    starting_index = index + 1;
                }
            }
            is_escaped = !is_escaped && c == '\\';
        }

        for range in &ranges_to_inspect {
            remove_comments_recrusive(line, Some(range));
        }

        if ranges_to_inspect.len() == 0 {
            remove_comments_recrusive(line, None);
        }

        line.is_empty()
    }

    pub fn remove_comments_recrusive(line: &mut String, range: Option<&(usize, usize)>) -> bool {
        if let Some(s) = line.find("/*") {
            if let Some((min, _)) = range { if s < *min {return false;}}
            let mut layer = 0;
            let mut last_char = ' ';
            for (index, c) in line.char_indices() {
                if layer == 1 && last_char == '*' && c == '/' {
                    if let Some((_, max)) = range { if index > *max {break;}}
                    line.replace_range(s..=index, "");
                    *line = line.trim().to_string();
                    break;
                } else if last_char == '*' && c == '/' {
                    layer -= 1;
                } else if last_char == '/' && c == '*' {
                    layer += 1;
                }

                last_char = c;
            }
        }
        if cut_off_between_strings("//", "\n", true, line, range) {
            remove_comments_recrusive(line, range);
        }

        line.is_empty()
    }

    pub fn remove_scopes(line: &mut String) -> bool {
        if let Some(last_index) = line.rfind('}') {
            line.replace_range(..=last_index, "");
            *line = line.trim().to_string();
            return false;
        } else if !line.contains('{') {return false;}

        true
    }

    fn remove_whitespace_from_attribute(line: &mut String) {
        let mut filtered_line = String::new();
        let mut is_in_quotes = false;
        let mut is_escaped = false;
        for c in line.chars() {
            if c == '"' && !is_escaped { is_in_quotes = !is_in_quotes; }
            is_escaped = !is_escaped && c == '\\';
            if is_in_quotes || !c.is_whitespace() {filtered_line.push(c);}
        }
        *line = filtered_line;

    }

    pub fn handle_attributes(line: &mut String) -> Option<PathBuf> {
        if !line.starts_with('#') { return None; }

        if let Some(last_bracket_index) = line.rfind(']'){
            let mut attributes: String = line.drain(0..=last_bracket_index).collect();
            remove_whitespace_from_attribute(&mut attributes);

            *line = line.trim().to_string();
            if let Some(mut path_start_index) = attributes.find("path=\"") {
                path_start_index += 6;
                let end_index = path_start_index + attributes[path_start_index..].find('"').unwrap();
                return Some(PathBuf::from(attributes[path_start_index..end_index].to_string()));
            }
        }

        None
    }