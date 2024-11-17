use scraper::{node::Element, ElementRef};
static URL: &str =
    "https://catalog.ucsc.edu/en/current/general-catalog/courses/am-applied-mathematics/";

#[derive(Debug)]
enum PE {
    E,
    H,
    T,
}
#[derive(Debug)]
enum PR {
    E,
    C,
    S,
}
#[derive(Debug)]
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

#[derive(Debug)]
struct Class {
    department: String,
    course_number: (u32, Option<char>),
    title: String,
    description: String,
    credits: u32,
    requirements: Option<Vec<String>>,
    gen_ed: Option<GenEds>,
    cross_listed: Option<String>,
}

fn main() {
    println!(
        "{:#?}",
        get_class_list(
            "https://catalog.ucsc.edu/en/current/general-catalog/courses/math-mathematics/"
        )
    );
    // get_class_list("https://catalog.ucsc.edu/en/current/general-catalog/courses/cse-computer-science-and-engineering");
}

fn get_class_list(url: &str) -> Vec<Class> {
    let resp = reqwest::blocking::get(url);
    let html = resp.unwrap().text().unwrap();

    let mut class_list: Vec<Class> = Vec::new();

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
                class_list.push(Class::new(curr_class_info));
            }
            curr_class_info = Vec::new();
        }
        curr_class_info.push(element);
    }

    class_list
}

impl Class {
    fn new(input_list: Vec<ElementRef>) -> Self {
        let class_dep_num_title = input_list[0].text().collect::<Vec<_>>();

        let (department, num) = class_dep_num_title[1].split_once(' ').unwrap_or_else(|| {
            eprintln!("Error parsing department and number: {:?}", input_list);
            panic!("Failed to parse input list");
        });

        let course_number: (u32, Option<char>) = if let Some(last_char) = num.chars().last() {
            if last_char.is_alphabetic() {
                (
                    num[..num.len() - 1].parse().expect("invalid course number"),
                    Some(last_char),
                )
            } else {
                (num.parse().expect("invalid course number"), None)
            }
        } else {
            panic!("Empty course number");
        };

        let description = input_list[1]
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        let department = department.to_string();

        let title = class_dep_num_title[2].to_string();

        let credits: u32 = input_list[2]
            .text()
            .collect::<Vec<_>>()
            .get(2)
            .expect("No credits data")
            .trim()
            .parse()
            .expect("Credits data not a number");

        let mut requirements = None;

        let mut gen_ed = None;

        let mut cross_listed = None;

        let mut input_iter = input_list.into_iter().skip(5).peekable();

        if let Some(element) = input_iter.peek() {
            //See if we have a crosslisted field
            if element.value().has_class(
                "crosslisted",
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            ) {
                cross_listed = input_iter.nth(2).map(|e| e.text().collect::<String>());
            }

            //See if we have a requirements field
            if let Some(element) = input_iter.peek() {
                if element
                    .value()
                    .has_class("instructor", scraper::CaseSensitivity::AsciiCaseInsensitive)
                {
                    input_iter.next();
                }
            }
            if let Some(element) = input_iter.peek() {
                //See if we have a crosslisted field
                if element.value().has_class(
                    "extraFields",
                    scraper::CaseSensitivity::AsciiCaseInsensitive,
                ) {
                    requirements = input_iter
                        .next()
                        .unwrap()
                        .text()
                        .collect::<String>()
                        .split_once(':')
                        .map(|(_, reqs)| {
                            reqs.split(';')
                                .map(str::trim)
                                .map(str::to_string)
                                .collect::<Vec<String>>()
                        });
                }
            }
            //See if we have gen_eds field
            if let Some(element) = input_iter.peek() {
                if element
                    .value()
                    .has_class("gen_ed", scraper::CaseSensitivity::AsciiCaseInsensitive)
                    || element
                        .value()
                        .has_class("genEd", scraper::CaseSensitivity::AsciiCaseInsensitive)
                {
                    gen_ed = GenEds::from_str(
                        input_iter
                            .next()
                            .unwrap()
                            .text()
                            .collect::<Vec<_>>()
                            .get(2)
                            .expect("Gen eds field missing"),
                    );
                }
            }
        }

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
    }
}

impl GenEds {
    fn from_str(input: &str) -> Option<GenEds> {
        match input {
            "MF" => Some(GenEds::MF),
            "CC" => Some(GenEds::CC),
            "ER" => Some(GenEds::ER),
            "IM" => Some(GenEds::IM),
            "SI" => Some(GenEds::SI),
            "SR" => Some(GenEds::SR),
            "TA" => Some(GenEds::TA),
            "C1" => Some(GenEds::C1),
            "C2" => Some(GenEds::C2),
            "PE-E" => Some(GenEds::PE(PE::E)),
            "PE-H" => Some(GenEds::PE(PE::H)),
            "PE-T" => Some(GenEds::PE(PE::T)),
            "PR-E" => Some(GenEds::PR(PR::E)),
            "PR-C" => Some(GenEds::PR(PR::C)),
            "PR-S" => Some(GenEds::PR(PR::S)),
            _ => None,
        }
    }
}
