use scraper::{node::Element, ElementRef};
static URL: &str =
    "https://catalog.ucsc.edu/en/current/general-catalog/courses/am-applied-mathematics/";

enum PE {
    E,
    H,
    T,
}
enum PR {
    E,
    C,
    S,
}
enum GenEds {
    MF,
    CC,
    ER,
    IM,
    SI,
    SR,
    TA,
    PE(PE),
    PR(PR),
    C1,
    C2,
}

struct Class {
    department: String,
    course_number: (u32, Option<char>),
    title: String,
    description: String,
    credits: u32,
    requirements: Option<Vec<String>>,
    gen_ed: Option<Vec<GenEds>>,
    cross_listed: Option<String>,
}

fn main() {
    let resp = reqwest::blocking::get(URL);
    let html = resp.unwrap().text().unwrap();

    let class_list: Vec<Class> = Vec::new();

    let selector = scraper::Selector::parse("div.courselist").unwrap();
    let document = scraper::Html::parse_document(&html);

    let course_list = document.select(&selector).next().unwrap();

    let mut curr_class_info = Vec::new();

    for element in course_list.child_elements() {
        if element.value().has_class(
            "course-name",
            scraper::CaseSensitivity::AsciiCaseInsensitive,
        ) {
            // println!("{:?}", curr_class_info);
            if !curr_class_info.is_empty() {
                Class::new(curr_class_info);
            }
            curr_class_info = Vec::new();
        }
        curr_class_info.push(element);
    }
}

impl Class {
    fn new(input_list: Vec<ElementRef>) -> Self {
        // println!("{:?}", input_list[0].text().collect::<Vec<_>>());
        let class_dep_num_title = input_list[0].text().collect::<Vec<_>>();

        let Some((department, num)) = class_dep_num_title[1].split_once(' ') else {
            eprintln!("something has gone horribly wrong");
            eprintln!("Here is a dump of the scraped list {:?}", input_list);
            panic!();
        };

        let course_number: (u32, Option<char>) = if num.chars().last().unwrap().is_alphabetic() {
            (
                num[..num.len() - 1].parse().unwrap(),
                Some(num.chars().last().unwrap()),
            )
        } else {
            (num.parse().unwrap(), None)
        };

        let binding = input_list[1].text().collect::<Vec<_>>().join("");
        let description = binding.trim().to_string();

        let department = department.to_string();

        let title = class_dep_num_title[2].to_string();

        let credits: u32 = input_list[2].text().collect::<Vec<_>>()[2]
            .trim()
            .parse()
            .unwrap();

        let mut index = 5;

        let cross_listed = match input_list[5].value().has_class(
            "crosslisted",
            scraper::CaseSensitivity::AsciiCaseInsensitive,
        ) {
            true => {
                index = 7;
                Some(input_list[6].text().collect::<String>())
            }
            false => None,
        };

        let requirements;

        let gen_ed;

        if index > input_list.len() {
            requirements = None;
            gen_ed = None;
        } else {
            requirements = match input_list[index].value().has_class(
                "extraFields",
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            ) {
                true => {
                    index += 1;
                    Some(
                        input_list[index - 1]
                            .text()
                            .collect::<String>()
                            .strip_prefix(':')
                            .unwrap()
                            .split(';')
                            .map(str::to_string)
                            .collect::<Vec<String>>(),
                    )
                }
                false => None,
            };
            if index > input_list.len() {
                gen_ed = match input_list[index]
                    .value()
                    .has_class("gen_ed", scraper::CaseSensitivity::AsciiCaseInsensitive)
                {
                    true => {
                        index += 1;
                        Some(input_list[index - 1].text().collect::<String>())
                    }
                    false => None,
                };
            }
        }

        println!("{:?}", input_list[index].text().collect::<Vec<_>>());
        Self {
            department,
            course_number,
            title,
            description,
            credits,
            requirements,
            gen_ed,
            cross_listed,
        }

        // println!("{:?}", input_list[5].text().collect::<Vec<_>>());
    }
}
