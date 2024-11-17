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

        let mut requirements = None;

        let mut gen_ed = None;

        let mut cross_listed = None;

        if index < input_list.len() {
            cross_listed = match input_list[5].value().has_class(
                "crosslisted",
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            ) {
                true => {
                    index = 7;
                    Some(input_list[6].text().collect::<String>())
                }
                false => None,
            };
            if index < input_list.len() {
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
                                .split_once(':')
                                .map(|tuple| {
                                    tuple
                                        .1
                                        .split(';')
                                        .map(str::trim)
                                        .map(str::to_string)
                                        .collect::<Vec<String>>()
                                }),
                        )
                    }
                    false => None,
                };
                if index < input_list.len() {
                    gen_ed = match input_list[index]
                        .value()
                        .has_class("gen_ed", scraper::CaseSensitivity::AsciiCaseInsensitive)
                        || input_list[index]
                            .value()
                            .has_class("genEd", scraper::CaseSensitivity::AsciiCaseInsensitive)
                    {
                        true => GenEds::from_str(input_list[index].text().collect::<Vec<_>>()[2]),
                        false => None,
                    };
                }
            }
        }

        Self {
            department,
            course_number,
            title,
            description,
            credits,
            requirements: requirements.flatten(),
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
