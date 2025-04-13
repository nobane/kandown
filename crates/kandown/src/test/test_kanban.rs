// crates/kandown/src/test/test_kanban.rs
//src/test/test_kanban.rs

use std::collections::HashMap;

use crate::{
    ColumnSort, Kanban, KanbanSortType, KanbanViewType, ParsedCard, ParsedDocument, ParsedProperty,
    ParsedPropertyType, ParsedPropertyValue, ParsedView, ParsedViewType, parse_cards_section,
    parse_view,
};

fn create_test_document() -> ParsedDocument {
    ParsedDocument {
        properties: vec![
            ParsedProperty {
                name: "Status".to_string(),
                property_type: ParsedPropertyType::Select {
                    options: vec![
                        "Backlog".to_string(),
                        "In Progress".to_string(),
                        "Done".to_string(),
                    ],
                },
            },
            ParsedProperty {
                name: "Owner".to_string(),
                property_type: ParsedPropertyType::Text,
            },
            ParsedProperty {
                name: "Due Date".to_string(),
                property_type: ParsedPropertyType::Date,
            },
        ],
        views: vec![
            ParsedView {
                name: "Board View".to_string(),
                layout: ParsedViewType::Board,
                group: Some("Status".to_string()),
                sort_by: None,
                sort_type: KanbanSortType::None,
                column_sorts: vec![],
                filter: None,
                display: Some("Owner".to_string()),
            },
            ParsedView {
                name: "Calendar View".to_string(),
                layout: ParsedViewType::Calendar,
                group: Some("Due Date".to_string()),
                sort_by: None,
                sort_type: KanbanSortType::None,
                column_sorts: vec![],
                filter: None,
                display: None,
            },
        ],
        cards: vec![
            ParsedCard {
                id: 0,
                title: "Task 1".to_string(),
                description: "Description for task 1".to_string(),
                properties: vec![
                    ParsedPropertyValue {
                        property_name: "Status".to_string(),
                        value: "Backlog".to_string(),
                    },
                    ParsedPropertyValue {
                        property_name: "Owner".to_string(),
                        value: "Alice".to_string(),
                    },
                ],
            },
            ParsedCard {
                id: 1,
                title: "Task 2".to_string(),
                description: "Description for task 2".to_string(),
                properties: vec![
                    ParsedPropertyValue {
                        property_name: "Status".to_string(),
                        value: "In Progress".to_string(),
                    },
                    ParsedPropertyValue {
                        property_name: "Owner".to_string(),
                        value: "Bob".to_string(),
                    },
                ],
            },
            ParsedCard {
                id: 2,
                title: "Task 3".to_string(),
                description: "Description for task 3".to_string(),
                properties: vec![
                    ParsedPropertyValue {
                        property_name: "Status".to_string(),
                        value: "Done".to_string(),
                    },
                    ParsedPropertyValue {
                        property_name: "Owner".to_string(),
                        value: "Charlie".to_string(),
                    },
                ],
            },
        ],
    }
}
#[test]
fn test_alpha_sort() {
    let mut doc = create_test_document();

    // Add more cards to make sorting more meaningful
    doc.cards.push(ParsedCard {
        id: 3,
        title: "ATask".to_string(),
        description: "Description for ATask".to_string(),
        properties: vec![
            ParsedPropertyValue {
                property_name: "Status".to_string(),
                value: "Backlog".to_string(),
            },
            ParsedPropertyValue {
                property_name: "Owner".to_string(),
                value: "Dave".to_string(),
            },
        ],
    });

    doc.cards.push(ParsedCard {
        id: 4,
        title: "ZTask".to_string(),
        description: "Description for ZTask".to_string(),
        properties: vec![
            ParsedPropertyValue {
                property_name: "Status".to_string(),
                value: "Backlog".to_string(),
            },
            ParsedPropertyValue {
                property_name: "Owner".to_string(),
                value: "Eve".to_string(),
            },
        ],
    });

    // Set the view to use alphabetical sorting
    doc.views[0].sort_type = KanbanSortType::Alpha;

    let board = Kanban::from_document(doc).unwrap();
    let grouped_cards = board.get_cards_by_group("Board View").unwrap();

    // Check the Backlog column is sorted alphabetically
    let backlog_cards = grouped_cards.get("Backlog").unwrap();
    assert_eq!(backlog_cards.len(), 3);
    assert_eq!(backlog_cards[0].borrow().title, "ATask");
    assert_eq!(backlog_cards[1].borrow().title, "Task 1");
    assert_eq!(backlog_cards[2].borrow().title, "ZTask");
}

#[test]
fn test_reverse_alpha_sort() {
    let mut doc = create_test_document();

    // Add more cards to make sorting more meaningful
    doc.cards.push(ParsedCard {
        id: 3,
        title: "ATask".to_string(),
        description: "Description for ATask".to_string(),
        properties: vec![
            ParsedPropertyValue {
                property_name: "Status".to_string(),
                value: "Backlog".to_string(),
            },
            ParsedPropertyValue {
                property_name: "Owner".to_string(),
                value: "Dave".to_string(),
            },
        ],
    });

    doc.cards.push(ParsedCard {
        id: 4,
        title: "ZTask".to_string(),
        description: "Description for ZTask".to_string(),
        properties: vec![
            ParsedPropertyValue {
                property_name: "Status".to_string(),
                value: "Backlog".to_string(),
            },
            ParsedPropertyValue {
                property_name: "Owner".to_string(),
                value: "Eve".to_string(),
            },
        ],
    });

    // Set the view to use reverse alphabetical sorting
    doc.views[0].sort_type = KanbanSortType::ReverseAlpha;

    let board = Kanban::from_document(doc).unwrap();
    let grouped_cards = board.get_cards_by_group("Board View").unwrap();

    // Check the Backlog column is sorted reverse alphabetically
    let backlog_cards = grouped_cards.get("Backlog").unwrap();
    assert_eq!(backlog_cards.len(), 3);
    assert_eq!(backlog_cards[0].borrow().title, "ZTask");
    assert_eq!(backlog_cards[1].borrow().title, "Task 1");
    assert_eq!(backlog_cards[2].borrow().title, "ATask");
}

#[test]
fn test_manual_sort() {
    let mut doc = create_test_document();

    // Set the view to use manual sorting
    doc.views[0].sort_type = KanbanSortType::Manual;

    // Define manual sorting for the Status columns
    doc.views[0].column_sorts = vec![
        ColumnSort {
            column: "Backlog".to_string(),
            order: vec![0], // Task 1
        },
        ColumnSort {
            column: "In Progress".to_string(),
            order: vec![1], // Task 2
        },
        ColumnSort {
            column: "Done".to_string(),
            order: vec![2], // Task 3
        },
    ];

    let board = Kanban::from_document(doc).unwrap();

    // Check the view has the manual sorting configuration
    let view = board.view_by_name.get("Board View").unwrap();
    assert_eq!(view.borrow().sort_type, KanbanSortType::Manual);
    assert_eq!(view.borrow().column_sorts.len(), 3);

    // Get the grouped cards
    let grouped_cards = board.get_cards_by_group("Board View").unwrap();

    // Check each column has the expected order
    let backlog_cards = grouped_cards.get("Backlog").unwrap();
    assert_eq!(backlog_cards.len(), 1);
    assert_eq!(backlog_cards[0].borrow().id, 0);
    assert_eq!(backlog_cards[0].borrow().title, "Task 1");

    let in_progress_cards = grouped_cards.get("In Progress").unwrap();
    assert_eq!(in_progress_cards.len(), 1);
    assert_eq!(in_progress_cards[0].borrow().id, 1);
    assert_eq!(in_progress_cards[0].borrow().title, "Task 2");
}

#[test]
fn test_parse_document_with_manual_sort() {
    // Test parsing a document with manual sort configuration
    let markdown = r#"# Properties
- Status: Select
	- Backlog
	- In Progress
	- Done

- Owner: Text

# Views
- Board View
  Layout: Board
  Group: Status
  Sort Type: Manual
    - Backlog
      - 2
      - 0
    - In Progress
      - 1

# Cards
- Task 1
  Status: Backlog
  Owner: Alice
  Description for task 1

- Task 2
  Status: In Progress
  Owner: Bob
  Description for task 2

- Task 3
  Status: Backlog
  Owner: Charlie
  Description for task 3
"#;

    let parsed_doc = ParsedDocument::try_from(markdown).unwrap();

    // Verify the board view has manual sorting
    assert_eq!(parsed_doc.views[0].sort_type, KanbanSortType::Manual);

    // Verify the column sorts were parsed correctly
    assert_eq!(parsed_doc.views[0].column_sorts.len(), 2);

    // Verify Backlog order
    let backlog_sort = parsed_doc.views[0]
        .column_sorts
        .iter()
        .find(|cs| cs.column == "Backlog")
        .unwrap();
    assert_eq!(backlog_sort.order, vec![2, 0]);

    // Verify In Progress order
    let in_progress_sort = parsed_doc.views[0]
        .column_sorts
        .iter()
        .find(|cs| cs.column == "In Progress")
        .unwrap();
    assert_eq!(in_progress_sort.order, vec![1]);

    // Create a board and check that the manual sorting works
    println!("{parsed_doc:#?}");
    let board = Kanban::from_document(parsed_doc).unwrap();
    let grouped_cards = board.get_cards_by_group("Board View").unwrap();

    // Backlog should have Task 1 first, then Task 3
    let backlog_cards = grouped_cards.get("Backlog").unwrap();
    assert_eq!(backlog_cards.len(), 2);
    assert_eq!(backlog_cards[0].borrow().title, "Task 3");
    assert_eq!(backlog_cards[1].borrow().title, "Task 1");
}

#[test]
fn test_roundtrip_with_sort_options() {
    // Test roundtrip with alpha sorting
    let markdown = r#"# Properties
- Status: Select
	- Backlog
	- In Progress
	- Done

- Owner: Text

# Views
- Board View
  Layout: Board
  Group: Status
  Sort Type: Alpha

# Cards
- Task 1
  Status: Backlog
  Owner: Alice

- Task 2
  Status: In Progress
  Owner: Bob
"#;

    let parsed_doc = ParsedDocument::try_from(markdown).unwrap();
    println!("{parsed_doc:#?}");
    let board = Kanban::from_document(parsed_doc).unwrap();
    let new_doc = board.to_parsed_document();
    let new_markdown = new_doc.to_string();

    // Ensure the Sort is preserved
    println!("{new_markdown}");
    assert!(new_markdown.contains("Sort Type: Alpha"));

    // Test roundtrip with manual sorting
    let markdown = r#"# Properties
- Status: Select
	- Backlog
	- In Progress
	- Done

- Owner: Text

# Views
- Board View
  Layout: Board
  Group: Status
  Sort Type: Manual
    - Backlog
      - 0
    - In Progress
      - 1

# Cards
- Task 1
  Status: Backlog
  Owner: Alice

- Task 2
  Status: In Progress
  Owner: Bob
"#;

    let parsed_doc = ParsedDocument::try_from(markdown).unwrap();
    let board = Kanban::from_document(parsed_doc).unwrap();
    let new_doc = board.to_parsed_document();
    let new_markdown = new_doc.to_string();

    // Ensure the Sort and column sorts are preserved
    assert!(new_markdown.contains("Sort Type: Manual"));
    assert!(new_markdown.contains("    - Backlog"));
    assert!(new_markdown.contains("      - 0"));
}

#[test]
fn test_card_id_system() {
    let doc = create_test_document();
    let board = Kanban::from_document(doc).unwrap();

    // Check that cards have the correct IDs
    let task1 = board.card_by_title.get("Task 1").unwrap();
    assert_eq!(task1.borrow().id, 0);

    let task2 = board.card_by_title.get("Task 2").unwrap();
    assert_eq!(task2.borrow().id, 1);

    let task3 = board.card_by_title.get("Task 3").unwrap();
    assert_eq!(task3.borrow().id, 2);

    // Check that card_by_id mapping works
    let card0 = board.card_by_id.get(&0).unwrap();
    assert_eq!(card0.borrow().title, "Task 1");

    let card1 = board.card_by_id.get(&1).unwrap();
    assert_eq!(card1.borrow().title, "Task 2");
}

#[test]
fn test_move_card_by_id() {
    let doc = create_test_document();
    let board = Kanban::from_document(doc).unwrap();

    // Move Task 1 (ID 0) from Backlog to In Progress using its ID
    board
        .move_card_by_id(0, "Board View", "In Progress")
        .unwrap();

    // Check card's property was updated
    let task1 = board.card_by_id.get(&0).unwrap();
    let status_value = task1
        .borrow()
        .properties
        .iter()
        .find(|(p, _)| p.borrow().name == "Status")
        .map(|(_, v)| v.clone())
        .unwrap();
    assert_eq!(status_value, "In Progress");

    // Check the card appears in correct group
    let grouped_cards = board.get_cards_by_group("Board View").unwrap();

    // Backlog should now be empty
    assert_eq!(grouped_cards.get("Backlog").unwrap().len(), 0);

    // In Progress should have 2 cards
    let in_progress_cards = grouped_cards.get("In Progress").unwrap();
    assert_eq!(in_progress_cards.len(), 2);

    // Both Task 1 and Task 2 should be in progress
    let ids: Vec<usize> = in_progress_cards.iter().map(|c| c.borrow().id).collect();
    assert!(ids.contains(&0)); // Task 1
    assert!(ids.contains(&1)); // Task 2
}

#[test]
fn test_to_parsed_document_with_sort_options() {
    let mut doc = create_test_document();

    // Set the view to use alphabetical sorting
    doc.views[0].sort_type = KanbanSortType::Alpha;

    let board = Kanban::from_document(doc.clone()).unwrap();

    // Convert back to ParsedDocument
    let parsed_doc = board.to_parsed_document();

    // Check that the sort type was preserved
    assert_eq!(parsed_doc.views[0].sort_type, KanbanSortType::Alpha);

    // Now test with manual sorting
    let mut doc = create_test_document();
    doc.views[0].sort_type = KanbanSortType::Manual;
    doc.views[0].column_sorts = vec![
        ColumnSort {
            column: "Backlog".to_string(),
            order: vec![0],
        },
        ColumnSort {
            column: "In Progress".to_string(),
            order: vec![1],
        },
    ];

    let board = Kanban::from_document(doc).unwrap();
    let parsed_doc = board.to_parsed_document();

    // Check that the sort type and column sorts were preserved
    assert_eq!(parsed_doc.views[0].sort_type, KanbanSortType::Manual);
    assert_eq!(parsed_doc.views[0].column_sorts.len(), 2);

    // Check column sorts
    let backlog_sort = parsed_doc.views[0]
        .column_sorts
        .iter()
        .find(|cs| cs.column == "Backlog")
        .unwrap();
    assert_eq!(backlog_sort.order, vec![0]);

    let in_progress_sort = parsed_doc.views[0]
        .column_sorts
        .iter()
        .find(|cs| cs.column == "In Progress")
        .unwrap();
    assert_eq!(in_progress_sort.order, vec![1]);
}

#[test]
fn test_kanban_board_creation() {
    let doc = create_test_document();
    let board = Kanban::from_document(doc).unwrap();

    // Verify properties
    assert_eq!(board.properties.len(), 3);
    assert_eq!(board.property_by_name.len(), 3);

    // Verify cards
    assert_eq!(board.cards.len(), 3);
    assert_eq!(board.card_by_title.len(), 3);

    // Verify views
    assert_eq!(board.views.len(), 2);
    assert_eq!(board.view_by_name.len(), 2);

    // Check property references
    let status_prop = board.property_by_name.get("Status").unwrap();
    assert_eq!(status_prop.borrow().name, "Status");
    assert_eq!(status_prop.borrow().cards.len(), 3);

    // Check card references
    let task1 = board.card_by_title.get("Task 1").unwrap();
    assert_eq!(task1.borrow().title, "Task 1");
    assert_eq!(task1.borrow().properties.len(), 2);
    assert_eq!(task1.borrow().views.len(), 2);

    // Check view references
    let board_view = board.view_by_name.get("Board View").unwrap();
    let board_view = board_view.borrow();
    assert_eq!(board_view.name, "Board View");
    match &board_view.view_layout {
        KanbanViewType::Board {
            group_by: Some(group_by),
        } => {
            assert_eq!(group_by.borrow().name, "Status")
        }
        _ => panic!(),
    };

    assert_eq!(board_view.cards.len(), 3);
}

#[test]
fn test_get_cards_by_group() {
    let doc = create_test_document();
    let board = Kanban::from_document(doc).unwrap();

    let grouped_cards = board.get_cards_by_group("Board View").unwrap();

    // Should have 3 groups (Backlog, In Progress, Done)
    assert_eq!(grouped_cards.len(), 3);

    // Check cards in each group
    assert_eq!(grouped_cards.get("Backlog").unwrap().len(), 1);
    assert_eq!(grouped_cards.get("In Progress").unwrap().len(), 1);
    assert_eq!(grouped_cards.get("Done").unwrap().len(), 1);

    // Check specific cards
    let backlog_cards = grouped_cards.get("Backlog").unwrap();
    assert_eq!(backlog_cards[0].borrow().title, "Task 1");

    let in_progress_cards = grouped_cards.get("In Progress").unwrap();
    assert_eq!(in_progress_cards[0].borrow().title, "Task 2");

    let done_cards = grouped_cards.get("Done").unwrap();
    assert_eq!(done_cards[0].borrow().title, "Task 3");
}

#[test]
fn test_add_card() {
    let doc = create_test_document();
    let mut board = Kanban::from_document(doc).unwrap();

    let mut properties = HashMap::new();
    properties.insert("Status".to_string(), "Backlog".to_string());
    properties.insert("Owner".to_string(), "Dave".to_string());

    let card_rc = board
        .add_card(
            "Task 4".to_string(),
            "Description for task 4".to_string(),
            properties,
        )
        .unwrap();

    // Check card was added to board
    assert_eq!(board.cards.len(), 4);
    assert!(board.card_by_title.contains_key("Task 4"));

    // Check card properties
    let card = card_rc.borrow();
    assert_eq!(card.title, "Task 4");
    assert_eq!(card.description, "Description for task 4");
    assert_eq!(card.properties.len(), 2);

    // Check the card was added to property references
    let status_prop = board.property_by_name.get("Status").unwrap();
    assert_eq!(status_prop.borrow().cards.len(), 4);

    // Check the card was added to views
    let board_view = board.view_by_name.get("Board View").unwrap();
    assert_eq!(board_view.borrow().cards.len(), 4);

    // Check the card appears in correct group
    let grouped_cards = board.get_cards_by_group("Board View").unwrap();
    let backlog_cards = grouped_cards.get("Backlog").unwrap();
    assert_eq!(backlog_cards.len(), 2);

    // One of the cards should be our new card
    let new_card_found = backlog_cards.iter().any(|c| c.borrow().title == "Task 4");
    assert!(new_card_found);
}

#[test]
fn test_move_card() {
    let doc = create_test_document();
    let board = Kanban::from_document(doc).unwrap();

    // Move Task 1 from Backlog to In Progress
    board
        .move_card("Task 1", "Board View", "In Progress")
        .unwrap();

    // Check card's property was updated
    let task1 = board.card_by_title.get("Task 1").unwrap();
    let status_value = task1
        .borrow()
        .properties
        .iter()
        .find(|(p, _)| p.borrow().name == "Status")
        .map(|(_, v)| v.clone())
        .unwrap();
    assert_eq!(status_value, "In Progress");

    // Check the card appears in correct group
    let grouped_cards = board.get_cards_by_group("Board View").unwrap();

    // Backlog should now be empty
    assert_eq!(grouped_cards.get("Backlog").unwrap().len(), 0);

    // In Progress should have 2 cards
    let in_progress_cards = grouped_cards.get("In Progress").unwrap();
    assert_eq!(in_progress_cards.len(), 2);

    // Both Task 1 and Task 2 should be in progress
    let titles: Vec<String> = in_progress_cards
        .iter()
        .map(|c| c.borrow().title.clone())
        .collect();
    assert!(titles.contains(&"Task 1".to_string()));
    assert!(titles.contains(&"Task 2".to_string()));
}

#[test]
fn test_to_parsed_document() {
    let doc = create_test_document();
    let board = Kanban::from_document(doc.clone()).unwrap();

    // Convert back to ParsedDocument
    let parsed_doc = board.to_parsed_document();

    // Properties should be the same
    assert_eq!(parsed_doc.properties.len(), doc.properties.len());
    for (i, prop) in parsed_doc.properties.iter().enumerate() {
        assert_eq!(prop.name, doc.properties[i].name);
        assert_eq!(prop.property_type, doc.properties[i].property_type);
    }

    // Cards should be the same
    assert_eq!(parsed_doc.cards.len(), doc.cards.len());
    for (i, card) in parsed_doc.cards.iter().enumerate() {
        assert_eq!(card.title, doc.cards[i].title);
        assert_eq!(card.description, doc.cards[i].description);
        assert_eq!(card.properties.len(), doc.cards[i].properties.len());
    }

    // Views should be the same
    assert_eq!(parsed_doc.views.len(), doc.views.len());
    for (i, view) in parsed_doc.views.iter().enumerate() {
        assert_eq!(view.name, doc.views[i].name);
        assert_eq!(view.layout, doc.views[i].layout);
        assert_eq!(view.group, doc.views[i].group);
    }
}

#[test]
fn test_invalid_property_reference() {
    let mut doc = create_test_document();

    // Add a card with a reference to a non-existent property
    doc.cards.push(ParsedCard {
        id: 4,
        title: "Invalid Card".to_string(),
        description: "This card has an invalid property".to_string(),
        properties: vec![ParsedPropertyValue {
            property_name: "NonExistentProperty".to_string(),
            value: "Some Value".to_string(),
        }],
    });

    // Should fail to create the board
    let result = Kanban::from_document(doc);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("unknown property"));
}

#[test]
fn test_invalid_select_value() {
    let doc = create_test_document();
    let mut board = Kanban::from_document(doc).unwrap();

    // Try to add a card with an invalid select value
    let mut properties = HashMap::new();
    properties.insert("Status".to_string(), "InvalidStatus".to_string());

    let result = board.add_card(
        "Invalid Card".to_string(),
        "This card has an invalid status".to_string(),
        properties,
    );

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("Invalid value for Status"));
}

#[test]
fn test_move_card_invalid_group() {
    let doc = create_test_document();
    let board = Kanban::from_document(doc).unwrap();

    // Try to move a card to an invalid group
    let result = board.move_card("Task 1", "Board View", "InvalidGroup");

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("Invalid group value"));
}

#[test]
fn test_view_not_found() {
    let doc = create_test_document();
    let board = Kanban::from_document(doc).unwrap();

    // Try to get cards for a non-existent view
    let result = board.get_cards_by_group("NonExistentView");

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("View not found"));
}

#[test]
fn test_view_without_group_by() {
    let mut doc = create_test_document();

    // Remove the group_by from one of the views
    doc.views[0].group = None;

    let board = Kanban::from_document(doc).unwrap();

    // Try to get cards grouped by a view without a group_by property
    let result = board.get_cards_by_group("Board View");

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("doesn't have a group_by property"));
}

#[test]
fn test_roundtrip_markdown() {
    // Add extra newlines between properties to ensure correct parsing
    let markdown = r#"# Properties
- Status: Select
	- Backlog
	- In Progress
	- Done

- Owner: Text

- Due Date: Date

# Views
- Board View
  Layout: Board
  Group: Status
  Display: Owner

- Calendar View
  Layout: Calendar
  Group: Due Date

# Cards
- Task 1
  Status: Backlog
  Owner: Alice
  Description for task 1

- Task 2
  Status: In Progress
  Owner: Bob
  Description for task 2

- Task 3
  Status: Done
  Owner: Charlie
  Description for task 3
"#;

    // Parse the markdown
    let parsed_doc = match ParsedDocument::try_from(markdown) {
        Ok(doc) => doc,
        Err(e) => {
            println!("Error parsing markdown: {}", e);
            panic!("Failed to parse markdown document");
        }
    };

    println!("{parsed_doc:#?}");

    // Verify the parsed document has the correct structure
    assert_eq!(
        parsed_doc.properties.len(),
        3,
        "Expected 3 properties, got: {:?}",
        parsed_doc.properties
    );

    // Create a board
    let board = Kanban::from_document(parsed_doc).unwrap();

    // Rest of the test remains the same...
    // Convert back to a parsed document
    let new_doc = board.to_parsed_document();

    // Convert to string
    let new_markdown = new_doc.to_string();

    // Parse the new markdown again
    let reparsed_doc = ParsedDocument::try_from(new_markdown.as_str()).unwrap();

    // Check properties
    assert_eq!(reparsed_doc.properties.len(), 3);

    // Check cards
    assert_eq!(reparsed_doc.cards.len(), 3);

    // Check views
    assert_eq!(reparsed_doc.views.len(), 2);

    // Create another board from the reparsed document
    let reparsed_board = Kanban::from_document(reparsed_doc).unwrap();

    // Check the grouped cards match
    let original_groups = board.get_cards_by_group("Board View").unwrap();
    let new_groups = reparsed_board.get_cards_by_group("Board View").unwrap();

    assert_eq!(original_groups.len(), new_groups.len());
    for (group, cards) in &original_groups {
        assert!(new_groups.contains_key(group));
        assert_eq!(cards.len(), new_groups.get(group).unwrap().len());
    }
}
