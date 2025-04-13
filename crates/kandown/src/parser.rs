// crates/kandown/src/parser.rs

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, line_ending, multispace0, not_line_ending, space0, space1},
    combinator::{map, opt, verify},
    error::{ErrorKind, ParseError},
    multi::{many0, many1},
    sequence::{preceded, terminated},
};

use crate::{
    ColumnSort, KanbanSortType, ParsedCard, ParsedDocument, ParsedProperty, ParsedPropertyType,
    ParsedPropertyValue, ParsedView, ParsedViewType,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownError<I> {
    NomError(I, ErrorKind),
    InvalidPropertyType(String),
    MissingSection(&'static str),
    InvalidFormat(String),
}

impl<I> ParseError<I> for MarkdownError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        MarkdownError::NomError(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

type ParserError<'a> = MarkdownError<&'a str>;
type ParserResult<'a, R> = IResult<&'a str, R, ParserError<'a>>;

mod parse_md {
    use super::*;

    pub fn heading<'a>(content: &str, i: &'a str) -> ParserResult<'a, ()> {
        map(
            terminated((char('#'), space0, tag(content)), opt(line_ending)),
            |_| (),
        )
        .parse(i)
    }

    pub fn list_item(i: &str) -> ParserResult<&str> {
        preceded(
            (
                space0,
                alt((
                    char('-'), // dash list item
                    char('*'), // asterisk list item
                    char('+'), // plus list item
                    preceded(
                        // numbered list item
                        take_while1(|c: char| c.is_ascii_digit()),
                        alt((char('.'), char(')'))),
                    ),
                )),
                space1,
            ),
            take_while1(|c: char| c != '\n' && c != '\r'),
        )
        .parse(i)
    }
}

// Document parser
pub(crate) fn parse_document(i: &str) -> ParserResult<ParsedDocument> {
    let (i, _) = multispace0.parse(i)?;

    let (i, properties) = match parse_properties_section(i) {
        Ok(result) => result,
        Err(nom::Err::Error(_)) => (i, vec![]), // Return empty vec on error
        Err(e) => return Err(e),                // Propagate other errors
    };

    let (i, views) = match parse_views_section(i) {
        Ok(result) => result,
        Err(nom::Err::Error(_)) => (i, vec![]), // Return empty vec on error
        Err(e) => return Err(e),                // Propagate other errors
    };

    // Parse cards and assign IDs based on position
    let (i, cards) = match parse_cards_section(i) {
        Ok(result) => result,
        Err(nom::Err::Error(_)) => (i, vec![]), // Return empty vec on error
        Err(e) => return Err(e),                // Propagate other errors
    };

    Ok((
        i,
        ParsedDocument {
            properties,
            cards,
            views,
        },
    ))
}

pub(crate) fn parse_card(i: &str) -> ParserResult<ParsedCard> {
    let (i, title) = parse_md::list_item(i)?;
    let (i, _) = opt(line_ending).parse(i)?;

    // Parse property values
    let mut property_values = Vec::new();
    let mut description = String::new();
    let mut current_input = i;

    // First, collect all property values
    loop {
        // We need to look ahead to see if this line has a colon (property) or not (description)
        if current_input.trim_start().starts_with('-')
            || current_input.trim_start().starts_with('#')
        {
            // This is a new item or section, we're done with this card
            break;
        }

        // Check if this line is indented and contains a colon (property)
        let mut indented_line = preceded(
            space1::<_, ParserError>,
            take_while1(|c: char| c != '\n' && c != '\r'),
        );
        if let Ok((_, line)) = indented_line.parse(current_input) {
            if line.contains(':') {
                // This is a property
                let (new_input, property_value) = parse_card_property(current_input)?;
                property_values.push(property_value);
                current_input = new_input;
            } else {
                // This is part of the description or we're done
                break;
            }
        } else {
            // No more indented lines, we're done
            break;
        }
    }

    // Now try to parse description (any indented text that's not a property)
    let mut description_parser = many0(preceded(
        space1::<_, ParserError>,
        map(
            terminated(
                verify(not_line_ending, |s: &str| !s.contains(':')),
                opt(line_ending),
            ),
            |s: &str| s.to_string(),
        ),
    ));

    if let Ok((new_input, desc_lines)) = description_parser.parse(current_input) {
        description = desc_lines.join("\n");
        current_input = new_input;
    }

    // Consume any trailing newlines
    let (current_input, _) = many0(line_ending).parse(current_input)?;

    Ok((
        current_input,
        ParsedCard {
            id: 0, // ID will be assigned later based on position
            title: title.trim().to_string(),
            description,
            properties: property_values,
        },
    ))
}

pub(crate) fn parse_card_property(i: &str) -> ParserResult<ParsedPropertyValue> {
    let (i, _) = space1.parse(i)?; // indentation

    // Parse name: value format
    let (i, line) = take_while1(|c: char| c != '\n' && c != '\r').parse(i)?;
    let (i, _) = opt(line_ending).parse(i)?;

    // Split by first colon
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(nom::Err::Error(MarkdownError::InvalidFormat(format!(
            "Invalid card property format: {}",
            line
        ))));
    }

    let property_name = parts[0].trim();
    let value = parts[1].trim();

    Ok((
        i,
        ParsedPropertyValue {
            property_name: property_name.to_string(),
            value: value.to_string(),
        },
    ))
}

pub(crate) fn parse_cards_section(i: &str) -> ParserResult<Vec<ParsedCard>> {
    let (i, _) = parse_md::heading("Cards", i)?;
    let (i, _) = many0(line_ending).parse(i)?; // Handle extra newlines

    // Parse cards first without assigning IDs
    let (i, mut cards) = many0(parse_card).parse(i)?;

    // Then explicitly assign IDs based on position
    for (idx, card) in cards.iter_mut().enumerate() {
        card.id = idx;
    }

    if cards.is_empty() {
        // If no cards were found, check if there might be a parsing issue
        if i.trim_start().starts_with('-') {
            // There are bullet points but we couldn't parse them as cards
            return Err(nom::Err::Error(MarkdownError::InvalidFormat(
                "Could not parse card items".to_string(),
            )));
        }
    }

    let (i, _) = many0(line_ending).parse(i)?;
    Ok((i, cards))
}

// Properties section parser
pub(crate) fn parse_properties_section(i: &str) -> ParserResult<Vec<ParsedProperty>> {
    let (i, _) = parse_md::heading("Properties", i)?;
    let (i, props) = many1(parse_property).parse(i)?;
    let (i, _) = many0(line_ending).parse(i)?;

    Ok((i, props))
}

// Property parser for the new format
pub(crate) fn parse_property(i: &str) -> ParserResult<ParsedProperty> {
    let (i, name_and_type) = parse_md::list_item(i)?;
    let (i, _) = opt(line_ending).parse(i)?;

    // Split the name and type by the colon
    let parts: Vec<&str> = name_and_type.split(':').collect();
    if parts.len() != 2 {
        return Err(nom::Err::Error(MarkdownError::InvalidFormat(format!(
            "Invalid property format: {name_and_type}",
        ))));
    }

    let name = parts[0].trim();
    let type_str = parts[1].trim();

    // Initialize property type based on the type string
    let (i, property_type) = match type_str {
        "Text" => (i, ParsedPropertyType::Text),
        "Number" => (i, ParsedPropertyType::Number),
        "Date" => (i, ParsedPropertyType::Date),
        "Checkbox" => (i, ParsedPropertyType::Checkbox),
        "Select" => {
            // For Select, we need to parse options
            let (i, options) = many0(parse_property_option).parse(i)?;
            (
                i,
                ParsedPropertyType::Select {
                    options: options.iter().map(|s| s.to_string()).collect(),
                },
            )
        }
        _ => {
            return Err(nom::Err::Error(MarkdownError::InvalidPropertyType(
                format!("Unknown property type: {type_str}"),
            )));
        }
    };

    let (i, _) = many0(line_ending).parse(i)?;

    Ok((
        i,
        ParsedProperty {
            name: name.to_string(),
            property_type,
        },
    ))
}

// Parse options for Select properties
pub(crate) fn parse_property_option(i: &str) -> ParserResult<&str> {
    let (i, _) = space0.parse(i)?; // indentation
    let (i, option) = parse_md::list_item(i)?; // option name
    let (i, _) = opt(line_ending).parse(i)?;

    Ok((i, option.trim()))
}

// Views section parser
pub(crate) fn parse_views_section(i: &str) -> ParserResult<Vec<ParsedView>> {
    let (i, _) = parse_md::heading("Views", i)?;
    let (i, views) = many1(parse_view).parse(i)?;
    let (i, _) = many0(line_ending).parse(i)?;

    Ok((i, views))
}

// Parse a view in the new format
pub(crate) fn parse_view(i: &str) -> ParserResult<ParsedView> {
    let (i, name) = parse_md::list_item(i)?;
    let (i, _) = opt(line_ending).parse(i)?;

    // Parse the view attributes
    let mut current_input = i;
    let mut view_type = None;
    let mut group_by = None;
    let mut filter = None;
    let mut sort_by = None;
    let mut sort_type = KanbanSortType::None;
    let mut display = None;
    let mut column_sorts = Vec::new();
    let mut current_column_sort: Option<ColumnSort> = None;

    // Parse each view attribute
    loop {
        // TODO: Make this more robust instead of trying to match "      " directly

        // Check if this is a deeper indented line (for manual sorting)
        if current_input.starts_with("      ") {
            if let Some(current_sort) = &mut current_column_sort {
                // This is a card ID under a column - must be a number
                match parse_number_option(current_input) {
                    Ok((new_input, id)) => {
                        current_sort.order.push(id);
                        let (after_line, _) = opt(line_ending).parse(new_input)?;
                        current_input = after_line;
                        continue;
                    }
                    Err(e) => return Err(e), // Return error if not a number
                }
            }
        }

        // Check if this is a column name for manual sorting
        if current_input.starts_with("    -") {
            // This is a column for manual sorting
            if let Ok((new_input, column_name)) =
                preceded(tag("    "), parse_md::list_item).parse(current_input)
            {
                // If we were already parsing a column, save it
                if let Some(sort) = current_column_sort.take() {
                    column_sorts.push(sort);
                }

                // Start a new column sort
                current_column_sort = Some(ColumnSort {
                    column: column_name.trim().to_string(),
                    order: Vec::new(),
                });

                let (after_line, _) = opt(line_ending).parse(new_input)?;
                current_input = after_line;
                continue;
            }
        }

        // Parse regular attribute lines
        let mut indented_line = preceded(
            space1::<_, ParserError>,
            take_while1(|c: char| c != '\n' && c != '\r'),
        );

        if let Ok((new_input, line)) = indented_line.parse(current_input) {
            // Try to split by colon to get attribute name and value
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                let attr_name = parts[0].trim();
                let attr_value = parts[1].trim();

                match attr_name {
                    "Layout" => {
                        view_type = Some(match attr_value {
                            "Board" => ParsedViewType::Board,
                            "Table" => ParsedViewType::Table,
                            "Calendar" => ParsedViewType::Calendar,
                            "Timeline" => ParsedViewType::Timeline,
                            _ => ParsedViewType::Board, // Default to Board for unknown types
                        });
                    }
                    "Group" => group_by = Some(attr_value.to_string()),
                    "Filter" => filter = Some(attr_value.to_string()),
                    "Sort" => sort_by = Some(attr_value.to_string()),
                    "Sort Type" => {
                        sort_type = match attr_value {
                            "Alpha" => KanbanSortType::Alpha,
                            "ReverseAlpha" => KanbanSortType::ReverseAlpha,
                            "Manual" => KanbanSortType::Manual,
                            _ => KanbanSortType::None,
                        };
                    }
                    "Display" => display = Some(attr_value.to_string()),
                    _ => {} // Ignore unknown attributes
                }
            }

            // Consume the line ending if present
            let (after_line, _) = opt(line_ending).parse(new_input)?;
            current_input = after_line;
        } else {
            // No more indented lines
            break;
        }
    }

    // Add the last column sort if we were parsing one
    if let Some(sort) = current_column_sort {
        column_sorts.push(sort);
    }

    // View type is required - default to Board if not specified
    let view_type = view_type.unwrap_or(ParsedViewType::Board);

    let (current_input, _) = many0(line_ending).parse(current_input)?;

    Ok((
        current_input,
        ParsedView {
            name: name.trim().to_string(),
            layout: view_type,
            group: group_by,
            filter,
            sort_by,
            sort_type,
            column_sorts,
            display,
        },
    ))
}
pub(crate) fn parse_number_option(i: &str) -> ParserResult<usize> {
    let (i, option) = parse_property_option(i)?;

    match option.trim().parse::<usize>() {
        Ok(num) => Ok((i, num)),
        Err(_) => Err(nom::Err::Error(MarkdownError::InvalidFormat(format!(
            "Expected number, got: {}",
            option
        )))),
    }
}
