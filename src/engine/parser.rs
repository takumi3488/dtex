use regex::Regex;
use std::{error::Error, fmt::Display, mem::take};
use yaml_rust::YamlLoader;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEOF,
    UnexpectedChar(char),
    UnsupportedIdentifier(String),
    YamlLoadError(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ParseError::UnexpectedEOF => "unexpected end of file".to_string(),
            ParseError::UnexpectedChar(c) => format!("unexpected character: {}", c),
            ParseError::UnsupportedIdentifier(s) => format!("unsupported identifier: {}", s),
            ParseError::YamlLoadError(s) => format!("yaml load error: {}", s),
        };
        write!(f, "ParseError: {}", msg)
    }
}

impl Error for ParseError {}

pub fn to_latex(expr: &str) -> Result<String, ParseError> {
    // 状態
    let mut in_equation = false;
    let mut in_align = false;
    let mut in_table = false;
    let mut decoration_stack = Vec::new();
    let mut seq = String::new();
    let mut table_seq = String::new();
    let mut get_table_caption = false;
    let mut in_front_matter = true;
    let mut front_matter = String::new();

    // 結果
    let mut res = vec![];

    // 正規表現
    let decorator_regex = Regex::new(r"^(@{1,2})([a-zA-Z][0-9a-zA-Z]+)(\s+\S+)*\s*$").unwrap();
    let empty_line_regex = Regex::new(r"^\s*$").unwrap();

    // 数式
    let equation_regex = Regex::new(r"^(equation|align)\*?$").unwrap();

    // 行ごとに処理
    let mut lines = expr.lines();
    while let Some(line) = lines.next() {
        // フロントマッターの処理
        if in_front_matter {
            front_matter.push_str(line);
            front_matter.push('\n');
            if line.starts_with("---") {
                let docs = YamlLoader::load_from_str(&front_matter).unwrap();
                let doc = &docs[0];

                // documentclassの処理
                res.push(format!(
                    "\\documentclass[a4paper,{},xelatex,ja=standard]{{bxjsarticle}}",
                    doc["config"]["fontsize"].as_str().unwrap_or("12pt")
                ));

                // packagesの処理
                for package in doc["config"]["packages"].as_vec().unwrap() {
                    let mut p = package.as_str().unwrap().split(".").collect::<Vec<&str>>();
                    p.reverse();
                    res.push(format!(
                        "\\usepackage{}{{{}}}",
                        &p[1..]
                            .iter()
                            .map(|s| format!("[{}]", s))
                            .collect::<Vec<String>>()
                            .join(""),
                        p[0]
                    ));
                }

                // titleの処理
                if !doc["cover"].is_badvalue() {
                    res.push(format!(
                        "\\title{{{}}}",
                        doc["cover"]["title"]
                            .as_str()
                            .ok_or(ParseError::YamlLoadError(
                                "title is not specified".to_string()
                            ))?
                    ));
                    res.push(format!(
                        "\\author{{{}}}",
                        doc["cover"]["author"]
                            .as_str()
                            .ok_or(ParseError::YamlLoadError(
                                "author is not specified".to_string()
                            ))?
                    ));
                    res.push(format!(
                        "\\date{{{}}}",
                        doc["cover"]["date"].as_str().unwrap_or(r"\today")
                    ));
                    res.push(r"\begin{document}".to_string());
                    res.push(r"\maketitle".to_string());
                } else {
                    res.push(r"\begin{document}".to_string());
                }

                in_front_matter = false;
            }
            continue;
        }

        // 本文の処理
        if line.contains("$$") {
            return Err(ParseError::UnsupportedIdentifier("$$".to_string()));
        } else if let Some(caps) = decorator_regex.captures(line) {
            // デコレーターの処理
            let type_ = caps.get(1).unwrap().as_str();
            let name = caps.get(2).unwrap().as_str();
            if equation_regex.is_match(name) {
                in_equation = true;
                in_align = name.starts_with("align")
            }
            let args = match caps.get(3) {
                Some(args) => args
                    .as_str()
                    .trim()
                    .split(" ")
                    .map(|s| format!("{{{}}}", s))
                    .collect::<Vec<String>>()
                    .join(""),
                None => String::new(),
            };
            // CSVの処理
            if name == "csv" {
                res.push(r"\begin{table}[hbtp]".to_string());
                res.push(r"\centering".to_string());
                res.push(format!(r"\begin{{tabular}}{}", args));
                in_table = true;
                get_table_caption = true;
                decoration_stack.push(name.to_string());
                continue;
            }
            if type_ == "@" {
                res.push(format!("\\{}{}", name, args));
            } else if type_ == "@@" {
                res.push(format!("\\begin{{{}}}{}", name, args));
                decoration_stack.push(name.to_string());
            } else {
                return Err(ParseError::UnexpectedChar(type_.chars().next().unwrap()));
            }
        } else if empty_line_regex.is_match(line) {
            // 空行の処理
            while decoration_stack.len() > 0 {
                let name = decoration_stack.pop().unwrap();

                // CSVの処理
                if name == "csv" {
                    let csv_seq = take(&mut table_seq);
                    let mut reader = csv::ReaderBuilder::new()
                        .has_headers(false)
                        .from_reader(csv_seq.as_bytes());
                    for (i, result) in reader.records().enumerate() {
                        let record = result.unwrap();
                        let row = record
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>()
                            .join(" & ");
                        res.push(format!("{} \\\\", row));
                        if i == 0 {
                            res.push(r"\hline \hline".to_string());
                        }
                    }
                    res.push(r"\hline".to_string());
                    res.push(r"\end{tabular}".to_string());
                    res.push(r"\end{table}".to_string());
                    in_table = false;
                    continue;
                }

                if equation_regex.is_match(&name) {
                    in_equation = false;
                    if name.starts_with("align") {
                        let last = res.pop().unwrap();
                        res.push(last.replace(r"\\", ""));
                        in_align = false;
                    }
                }
                res.push(format!("\\end{{{}}}", name));
            }
        } else {
            // 文字ごとに処理
            let mut chars = line.chars();
            while let Some(c) = chars.next() {
                match (c, in_equation) {
                    ('$', false) => {
                        in_equation = true;
                        seq.push(c);
                    }
                    ('$', true) => {
                        in_equation = false;
                        seq.push(c);
                    }
                    ('@', true) => {
                        let mut inl = String::new();
                        let mut c = chars.next().ok_or(ParseError::UnexpectedEOF)?;
                        while c != '@' {
                            if c == '$' {
                                Err(ParseError::UnexpectedChar(c))?;
                            }
                            inl.push(c);
                            c = chars.next().ok_or(ParseError::UnexpectedEOF)?;
                        }
                        let name = inl.split(" ").next().unwrap();
                        let args: &str = &inl
                            .split(" ")
                            .skip(1)
                            .map(|s| format!("{{{}}}", s))
                            .collect::<Vec<String>>()
                            .join("");
                        seq.push_str(&format!("\\{}{}", name, args));
                    }
                    (c, _) => {
                        if in_align && c == '=' {
                            seq.push('&');
                        }
                        seq.push(c);
                    }
                }
            }
            if in_table {
                if get_table_caption {
                    let tmp = res.pop().unwrap();
                    res.push(format!("\\caption{{{}}}", take(&mut seq)));
                    res.push(tmp);
                    res.push(r"\hline".to_string());
                    get_table_caption = false;
                    continue;
                }
                let row = take(&mut seq);
                table_seq.push_str(&row);
                table_seq.push('\n');
                continue;
            }
            if in_align {
                seq.push_str(r"\\");
            }
            res.push(take(&mut seq));
        }
    }
    res.push(r"\end{document}".to_string());

    Ok(res.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expr_with_cover() {
        let expr = r#"config:
    fontsize: 11pt
    packages:
        - amsmath
cover:
    title: "Example"
    author: "Author"
    date: "2020-01-01"
---
@@align
e^{i\pi} = @cos \pi@ + i@sin \pi@
=-1

@@csv ccc
example table
test1,test2,test3
$@SI 163 cm@$,$@SI 171 cm@$,$@SI 178 cm@$

"#;
        let got = to_latex(expr).unwrap();
        let want = r#"\documentclass[a4paper,11pt,xelatex,ja=standard]{bxjsarticle}
\usepackage{amsmath}
\title{Example}
\author{Author}
\date{2020-01-01}
\begin{document}
\maketitle
\begin{align}
e^{i\pi} &= \cos{\pi} + i\sin{\pi}\\
&=-1
\end{align}
\begin{table}[hbtp]
\centering
\caption{example table}
\begin{tabular}{ccc}
\hline
test1 & test2 & test3 \\
\hline \hline
$\SI{163}{cm}$ & $\SI{171}{cm}$ & $\SI{178}{cm}$ \\
\hline
\end{tabular}
\end{table}
\end{document}"#;
        assert_eq!(got, want);
    }

    #[test]
    fn test_parse_expr_without_cover() {
        let expr = r#"config:
    fontsize: 11pt
    packages:
        - amsmath
---
@@align
e^{i\pi} = @cos \pi@ + i@sin \pi@
=-1

@@csv ccc
example table
test1,test2,test3
$@SI 163 cm@$,$@SI 171 cm@$,$@SI 178 cm@$

"#;
        let got = to_latex(expr).unwrap();
        let want = r#"\documentclass[a4paper,11pt,xelatex,ja=standard]{bxjsarticle}
\usepackage{amsmath}
\begin{document}
\begin{align}
e^{i\pi} &= \cos{\pi} + i\sin{\pi}\\
&=-1
\end{align}
\begin{table}[hbtp]
\centering
\caption{example table}
\begin{tabular}{ccc}
\hline
test1 & test2 & test3 \\
\hline \hline
$\SI{163}{cm}$ & $\SI{171}{cm}$ & $\SI{178}{cm}$ \\
\hline
\end{tabular}
\end{table}
\end{document}"#;
        assert_eq!(got, want);
    }
}
