// crates/kandown/src/parsed_document.rs

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParsedViewType {
    Board,
    Table,
    Calendar,
    Timeline,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParsedPropertyType {
    Text,
    Select { options: Vec<String> },
    Number,
    Date,
    Checkbox,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedProperty {
    pub name: String,
    pub property_type: ParsedPropertyType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedPropertyValue {
    pub property_name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedCard {
    pub id: usize, // Added ID based on position
    pub title: String,
    pub description: String,
    pub properties: Vec<ParsedPropertyValue>,
}

impl std::fmt::Display for ParsedViewType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ParsedViewType::Board => write!(f, "Board",),
            ParsedViewType::Table => write!(f, "Table",),
            ParsedViewType::Calendar => write!(f, "Calendar",),
            ParsedViewType::Timeline => write!(f, "Timeline",),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum KanbanSortType {
    Alpha,
    ReverseAlpha,
    Manual,
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnSort {
    pub column: String,
    pub order: Vec<usize>, // Card IDs in order
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedView {
    pub name: String,
    pub layout: ParsedViewType,
    pub group: Option<String>,
    pub filter: Option<String>,
    pub sort_by: Option<String>,
    pub sort_type: KanbanSortType,
    pub column_sorts: Vec<ColumnSort>, // For manual sorting
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedDocument {
    pub properties: Vec<ParsedProperty>,
    pub cards: Vec<ParsedCard>,
    pub views: Vec<ParsedView>,
}

impl std::convert::TryFrom<&str> for ParsedDocument {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ParsedDocument::try_parse(value).map(|(_, doc)| doc)
    }
}

impl ParsedDocument {
    pub fn try_parse(s: &str) -> anyhow::Result<(&str, ParsedDocument)> {
        super::parse_document(s).map_err(|e| anyhow!("{e}"))
    }
}

impl std::str::FromStr for ParsedDocument {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ParsedDocument::try_from(s)
    }
}

impl std::fmt::Display for ParsedDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        // Properties section
        if !self.properties.is_empty() {
            output.push_str("# Properties\n");

            for ParsedProperty {
                name,
                property_type,
            } in &self.properties
            {
                match &property_type {
                    ParsedPropertyType::Text => {
                        output.push_str(&format!("- {name}: Text\n"));
                    }
                    ParsedPropertyType::Date => {
                        output.push_str(&format!("- {name}: Date\n"));
                    }
                    ParsedPropertyType::Number => {
                        output.push_str(&format!("- {name}: Number\n"));
                    }
                    ParsedPropertyType::Checkbox => {
                        output.push_str(&format!("- {name}: Checkbox\n"));
                    }
                    ParsedPropertyType::Select { options } => {
                        output.push_str(&format!("- {name}: Select\n"));
                        for option in options {
                            output.push_str(&format!("\t- {option}\n"));
                        }
                    }
                }

                output.push('\n');
            }
        }

        // Views section
        if !self.views.is_empty() {
            output.push_str("# Views\n");

            for ParsedView {
                name,
                layout,
                group,
                filter,
                sort_by,
                sort_type,
                column_sorts,
                display,
            } in &self.views
            {
                output.push_str(&format!("- {name}\n"));

                output.push_str(&format!("  Layout: {layout}\n"));

                if let Some(group_by) = &group {
                    output.push_str(&format!("  Group: {group_by}\n"));
                }

                if let Some(sort_by) = &sort_by {
                    output.push_str(&format!("  Sort: {sort_by}\n"));
                }

                // Output sort type if it's not None
                if sort_type != &KanbanSortType::None {
                    output.push_str(&format!("  Sort Type: {sort_type:?}\n"));
                }

                // Output manual sorting configuration if present
                if sort_type == &KanbanSortType::Manual && !column_sorts.is_empty() {
                    for ColumnSort { column, order } in column_sorts {
                        output.push_str(&format!("    - {column}\n"));
                        for id in order {
                            output.push_str(&format!("      - {id}\n"));
                        }
                    }
                }

                if let Some(filter) = &filter {
                    output.push_str(&format!("  Filter: {filter}\n"));
                }

                if let Some(display) = &display {
                    output.push_str(&format!("  Display: {display}\n"));
                }

                output.push('\n');
            }
        }

        // Cards section
        if !self.cards.is_empty() {
            output.push_str("# Cards\n");

            for ParsedCard {
                title,
                description,
                properties,
                ..
            } in &self.cards
            {
                output.push_str(&format!("- {title}\n"));

                for ParsedPropertyValue {
                    property_name,
                    value,
                } in properties
                {
                    output.push_str(&format!("  {property_name}: {value}\n"));
                }

                if !description.is_empty() {
                    output.push_str(&format!("  {description}\n"));
                }

                output.push('\n');
            }
        }

        write!(f, "{output}")
    }
}
