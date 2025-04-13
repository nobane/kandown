// crates/kandown/src/test/test_parser.rs
use crate::*;

#[test]
fn test_property_parser() {
    let input = "- Owner: Text\n";
    let (_rest, property) = parse_property(input).unwrap();
    assert_eq!(property.name, "Owner");
    assert_eq!(property.property_type, ParsedPropertyType::Text);

    let input = "- Status: Select\n\t- Backlog\n\t- In Progress\n\t- Done\n";
    let (_rest, property) = parse_property(input).unwrap();
    assert_eq!(property.name, "Status");
    assert_eq!(
        property.property_type,
        ParsedPropertyType::Select {
            options: vec![
                "Backlog".to_string(),
                "In Progress".to_string(),
                "Done".to_string()
            ]
        }
    );
}

#[test]
fn test_card_parser() {
    let input = "- Task 1\n  Created By: bob\n  Status: In Progress\n  This is a description\n  With multiple lines\n";
    let (_rest, card) = parse_card(input).unwrap();
    assert_eq!(card.title, "Task 1");
    assert_eq!(
        card.description,
        "This is a description\nWith multiple lines"
    );
    assert_eq!(card.properties.len(), 2);
    assert_eq!(card.properties[0].property_name, "Created By");
    assert_eq!(card.properties[0].value, "bob");
    assert_eq!(card.properties[1].property_name, "Status");
    assert_eq!(card.properties[1].value, "In Progress");
}

#[test]
fn test_view_parser() {
    let input = "- Task Board\n  Layout: Board\n  Group: Status\n";
    let (_rest, view) = parse_view(input).unwrap();
    assert_eq!(view.name, "Task Board");
    assert_eq!(view.layout, ParsedViewType::Board);
    assert_eq!(view.group, Some("Status".to_string()));
}

#[test]
fn test_full_document() {
    let input = r#"
# Properties
- Owner: Text
- Status: Select
	- Backlog
	- In Progress
	- Done

# Views
- Task Board
  Layout: Board
  Group: Status
- Timeline
  Layout: Calendar
  Group: Due Date

# Cards
- Task 1
  Owner: bob
  Status: In Progress
  This is a description
- Task 2
  Owner: alice
  Status: Backlog
  Another task description
"#;
    let (_rest, doc) = parse_document(input).unwrap();
    println!("{doc:?}");
    // Verify properties
    assert_eq!(doc.properties.len(), 2);
    assert_eq!(doc.properties[0].name, "Owner");
    assert_eq!(doc.properties[1].name, "Status");

    // Verify cards
    assert_eq!(doc.cards.len(), 2);
    assert_eq!(doc.cards[0].title, "Task 1");
    assert_eq!(doc.cards[0].properties[0].value, "bob");
    assert_eq!(doc.cards[1].title, "Task 2");

    // Verify views
    assert_eq!(doc.views.len(), 2);
    assert_eq!(doc.views[0].name, "Task Board");
    assert_eq!(doc.views[1].name, "Timeline");

    // Test serialization
    let output = doc.to_string();
    assert!(output.contains("# Properties"));
    assert!(output.contains(": Select"));
    assert!(output.contains("\t- In Progress"));
    assert!(output.contains("Task 1"));
    assert!(output.contains("Owner: bob"));
    assert!(output.contains("This is a description"));
    assert!(output.contains("Layout: Board"));
}

#[test]
fn test_roundtrip() {
    let input = r#"# Properties
- Owner: Text
- Status: Select
	- Backlog
	- In Progress
	- Done

# Views
- Task Board
  Layout: Board
  Group: Status

# Cards
- Task 1
  Owner: bob
  Status: In Progress
  This is a description

- Task 2
  Owner: alice
  Status: Backlog
  Another task description
"#;
    // Parse the document
    let (_, doc) = parse_document(input).unwrap();

    // Serialize it back
    let output = doc.to_string();

    // Parse the serialized output again
    let (_, doc2) = parse_document(&output).unwrap();

    // They should be equal
    assert_eq!(doc.properties.len(), doc2.properties.len());
    assert_eq!(doc.cards.len(), doc2.cards.len());
    assert_eq!(doc.views.len(), doc2.views.len());

    // Check some specific values
    assert_eq!(doc.cards[0].title, doc2.cards[0].title);
    assert_eq!(doc.views[0].layout, doc2.views[0].layout);
}

#[test]
fn test_custom_property_types() {
    let input = r#"# Properties
- Due Date: Date
- Priority: Number
- Completed: Checkbox

# Cards
- Task 1
  Due Date: 2023-09-30
  Priority: 1
  Completed: true
  Important task

# Views
- Calendar View
  Layout: Calendar
  Group: Due Date
"#;

    let (_, doc) = parse_document(input).unwrap();

    assert_eq!(doc.properties.len(), 3);
    assert_eq!(doc.properties[0].property_type, ParsedPropertyType::Date);
    assert_eq!(doc.properties[1].property_type, ParsedPropertyType::Number);
    assert_eq!(
        doc.properties[2].property_type,
        ParsedPropertyType::Checkbox
    );

    assert_eq!(doc.cards[0].properties[0].property_name, "Due Date");
    assert_eq!(doc.cards[0].properties[0].value, "2023-09-30");

    let output = doc.to_string();
    assert!(output.contains(": Date"));
    assert!(output.contains(": Number"));
    assert!(output.contains(": Checkbox"));
}

#[test]
fn test_example_from_prompt() {
    let input = r#"# Properties
- Created By: Text
- Due Date: Date
- Status: Select
	- Backlog
	- In Progress
	- Done

# Views
- Main Board
  Layout: Board
  Group: Status
  Sort: Manual
  Display: Created By

- Deadline Calendar
  Layout: Calendar
  Group: Due Date

# Cards
- Some Card
  Created By: Bob
  Status: Backlog
  This is a description of the card

- Another Card
  Created By: Jane
  Status: Backlog

- And another
  Created By: Jane
  Status: Done

- Build It
  Created By: Bob
  Status: In Progress
"#;

    let (_, doc) = parse_document(input).unwrap();

    // Check properties
    assert_eq!(doc.properties.len(), 3);
    assert_eq!(doc.properties[0].name, "Created By");
    assert_eq!(doc.properties[0].property_type, ParsedPropertyType::Text);
    assert_eq!(doc.properties[1].name, "Due Date");
    assert_eq!(doc.properties[1].property_type, ParsedPropertyType::Date);

    // Check views
    assert_eq!(doc.views.len(), 2);
    assert_eq!(doc.views[0].name, "Main Board");
    assert_eq!(doc.views[0].layout, ParsedViewType::Board);
    assert_eq!(doc.views[0].sort_by, Some("Manual".to_string()));
    assert_eq!(doc.views[0].display, Some("Created By".to_string()));

    // Check cards
    assert_eq!(doc.cards.len(), 4);
    assert_eq!(doc.cards[0].title, "Some Card");
    assert_eq!(
        doc.cards[0].description,
        "This is a description of the card"
    );
    assert_eq!(doc.cards[3].title, "Build It");
    assert_eq!(doc.cards[3].properties[1].value, "In Progress");
}

#[test]
fn test_parse_sort_options() {
    // Test parsing a view with Alpha sort
    let input = r#"- Task Board
  Layout: Board
  Group: Status
  Sort Type: Alpha
"#;
    let (_rest, view) = parse_view(input).unwrap();
    assert_eq!(view.name, "Task Board");
    assert_eq!(view.layout, ParsedViewType::Board);
    assert_eq!(view.sort_type, KanbanSortType::Alpha);

    // Test parsing a view with ReverseAlpha sort
    let input = r#"- Task Board
  Layout: Board
  Group: Status
  Sort Type: ReverseAlpha
"#;
    let (_rest, view) = parse_view(input).unwrap();
    assert_eq!(view.sort_type, KanbanSortType::ReverseAlpha);

    // Test parsing a view with Manual sort
    let input = r#"- Task Board
  Layout: Board
  Group: Status
  Sort Type: Manual
    - Backlog
      - 1
      - 0
    - In Progress
      - 2
"#;
    let (_rest, view) = parse_view(input).unwrap();
    assert_eq!(view.sort_type, KanbanSortType::Manual);
    assert_eq!(view.column_sorts.len(), 2);

    // Check the column sorts
    let backlog_sort = view
        .column_sorts
        .iter()
        .find(|cs| cs.column == "Backlog")
        .unwrap();
    assert_eq!(backlog_sort.order, vec![1, 0]);

    let in_progress_sort = view
        .column_sorts
        .iter()
        .find(|cs| cs.column == "In Progress")
        .unwrap();
    assert_eq!(in_progress_sort.order, vec![2]);

    // Test parsing a view with Manual sort
    let input = r#"- Task Board
  Layout: Board
  Group: Status
  Sort Type: Manual
    - Backlog
      - 0
      - 1
    - In Progress
      - 2
"#;
    let (_rest, view) = parse_view(input).unwrap();
    assert_eq!(view.sort_type, KanbanSortType::Manual);
    assert_eq!(view.column_sorts.len(), 2);

    // Check the column sorts
    let backlog_sort = view
        .column_sorts
        .iter()
        .find(|cs| cs.column == "Backlog")
        .unwrap();
    assert_eq!(backlog_sort.order, vec![0, 1]);

    let in_progress_sort = view
        .column_sorts
        .iter()
        .find(|cs| cs.column == "In Progress")
        .unwrap();
    assert_eq!(in_progress_sort.order, vec![2]);
}

#[test]
fn test_parse_document_with_card_ids() {
    let input = r#"# Cards
- Task 1
  Status: Backlog
  Description for task 1

- Task 2
  Status: In Progress
  Description for task 2
"#;
    let (_rest, cards) = parse_cards_section(input).unwrap();

    // Verify cards have IDs assigned based on position
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].id, 0);
    assert_eq!(cards[1].id, 1);
}

#[test]
fn test_parse_invalid_manual_sort() {
    // Test parsing with invalid manual sort entries (non-numeric)
    let input = r#"- Task Board
  Layout: Board
  Group: Status
  Sort: Manual
    - Backlog
      - abc
"#;

    let result = parse_view(input);
    assert!(result.is_err());
}
