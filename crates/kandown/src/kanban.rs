// crates/kandown/src/kanban.rs

use anyhow::{Result, anyhow, bail};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    ColumnSort, KanbanSortType, ParsedCard, ParsedDocument, ParsedProperty, ParsedPropertyType,
    ParsedPropertyValue, ParsedView, ParsedViewType,
};

pub struct Kanban {
    pub properties: Vec<Rc<RefCell<KanbanProperty>>>,
    pub views: Vec<Rc<RefCell<KanbanView>>>,
    pub cards: Vec<Rc<RefCell<KanbanCard>>>,

    pub property_by_name: HashMap<String, Rc<RefCell<KanbanProperty>>>,
    pub card_by_title: HashMap<String, Rc<RefCell<KanbanCard>>>,
    pub card_by_id: HashMap<usize, Rc<RefCell<KanbanCard>>>,
    pub view_by_name: HashMap<String, Rc<RefCell<KanbanView>>>,
}

pub enum KanbanPropertyType {
    Text,
    Number,
    Date,
    Checkbox,
    Select { options: Vec<String> },
}

pub struct KanbanProperty {
    pub name: String,
    pub property_type: KanbanPropertyType,
    pub cards: Vec<Weak<RefCell<KanbanCard>>>, // Reference to all cards that use this property
}

pub struct KanbanCard {
    pub id: usize, // Added ID based on position
    pub title: String,
    pub description: String,
    pub properties: Vec<(Rc<RefCell<KanbanProperty>>, String)>,
    pub views: Vec<Weak<RefCell<KanbanView>>>, // Weak references to avoid cycles
}

pub enum KanbanViewType {
    Board {
        group_by: Option<Rc<RefCell<KanbanProperty>>>,
    },
    Table {
        sort_by: Option<Rc<RefCell<KanbanProperty>>>,
    },
    Calendar {
        date_property: Option<Rc<RefCell<KanbanProperty>>>,
    },
    Timeline {
        date_property: Option<Rc<RefCell<KanbanProperty>>>,
    },
}

pub struct KanbanView {
    pub name: String,
    pub view_layout: KanbanViewType,
    pub filter: Option<String>,
    pub sort_type: KanbanSortType,
    pub sort_by: Option<Rc<RefCell<KanbanProperty>>>,
    pub column_sorts: HashMap<String, Vec<usize>>, // For manual sorting
    pub display: Vec<Rc<RefCell<KanbanProperty>>>,
    pub cards: Vec<Rc<RefCell<KanbanCard>>>,
}

impl Kanban {
    pub fn from_document(doc: ParsedDocument) -> Result<Self> {
        // First pass: create all the properties
        let mut properties = Vec::new();
        let mut property_by_name = HashMap::new();

        for prop in doc.properties {
            let property_type = match prop.property_type {
                ParsedPropertyType::Text => KanbanPropertyType::Text,
                ParsedPropertyType::Number => KanbanPropertyType::Number,
                ParsedPropertyType::Date => KanbanPropertyType::Date,
                ParsedPropertyType::Checkbox => KanbanPropertyType::Checkbox,
                ParsedPropertyType::Select { options } => KanbanPropertyType::Select {
                    options: options.clone(),
                },
            };

            let prop_rc = Rc::new(RefCell::new(KanbanProperty {
                name: prop.name.clone(),
                property_type,
                cards: Vec::new(),
            }));

            properties.push(Rc::clone(&prop_rc));
            property_by_name.insert(prop.name, prop_rc);
        }

        // Second pass: create all the cards and link to properties
        let mut cards = Vec::new();
        let mut card_by_title = HashMap::new();
        let mut card_by_id = HashMap::new();

        for parsed_card in doc.cards {
            let mut card_properties = Vec::new();

            // Link card to its properties
            for prop_value in &parsed_card.properties {
                let prop_name = &prop_value.property_name;
                let prop_rc = property_by_name
                    .get(prop_name)
                    .ok_or_else(|| anyhow!("Card references unknown property: {prop_name}"))?;

                card_properties.push((Rc::clone(prop_rc), prop_value.value.clone()));
            }

            let card_rc = Rc::new(RefCell::new(KanbanCard {
                id: parsed_card.id,
                title: parsed_card.title.clone(),
                description: parsed_card.description,
                properties: card_properties,
                views: Vec::new(),
            }));

            cards.push(Rc::clone(&card_rc));
            card_by_title.insert(parsed_card.title, Rc::clone(&card_rc));
            card_by_id.insert(parsed_card.id, Rc::clone(&card_rc));
        }

        // Update property->card references
        for card_rc in &cards {
            let card = card_rc.borrow();

            for (prop_rc, _) in &card.properties {
                let weak_card = Rc::downgrade(card_rc);
                prop_rc.borrow_mut().cards.push(weak_card);
            }
        }

        // Third pass: create views and link to properties and cards
        let mut views = Vec::new();
        let mut view_by_name = HashMap::new();

        for ParsedView {
            name,
            layout: view_type,
            sort_by,
            group,
            filter,
            sort_type,
            column_sorts,
            display,
        } in doc.views
        {
            let mut view_properties = Vec::new();
            let mut column_sorts_map = HashMap::new();

            // Convert column_sorts to HashMap
            for column_sort in column_sorts {
                column_sorts_map.insert(column_sort.column, column_sort.order);
            }

            // Link sort_by property if present
            let mut sort_by_prop = None;
            if let Some(sort_by_name) = &sort_by {
                if let Some(prop_rc) = property_by_name.get(sort_by_name) {
                    sort_by_prop = Some(Rc::clone(prop_rc));
                } else {
                    println!(
                        "Warning: View '{name}' references unknown sort_by property: {sort_by_name}"
                    );
                }
            }

            // Set up view type with appropriate properties
            let kanban_view_type = match view_type {
                ParsedViewType::Board => {
                    let mut group_by_prop = None;

                    // Link group_by property
                    if let Some(group_name) = &group {
                        if let Some(prop_rc) = property_by_name.get(group_name) {
                            group_by_prop = Some(Rc::clone(prop_rc));
                        } else {
                            bail!("View '{name}' references unknown group by: {group_name}")
                        }
                    }

                    KanbanViewType::Board {
                        group_by: group_by_prop,
                    }
                }
                ParsedViewType::Table => {
                    let mut sort_by_prop = None;

                    // Link sort_by property
                    if let Some(sort_by_name) = &sort_by {
                        if let Some(prop_rc) = property_by_name.get(sort_by_name) {
                            sort_by_prop = Some(Rc::clone(prop_rc));
                        } else {
                            bail!("View '{name}' references unknown sort_by: {sort_by_name}")
                        }
                    }

                    KanbanViewType::Table {
                        sort_by: sort_by_prop,
                    }
                }
                ParsedViewType::Calendar => {
                    let mut date_property = None;

                    // Link date property (using group as the date field)
                    if let Some(date_field) = &group {
                        if let Some(prop_rc) = property_by_name.get(date_field) {
                            date_property = Some(Rc::clone(prop_rc));
                        } else {
                            bail!("View '{name}' references unknown date property: {date_field}")
                        }
                    }

                    KanbanViewType::Calendar { date_property }
                }
                ParsedViewType::Timeline => {
                    let mut date_property = None;

                    // Link date property (using group as the date field)
                    if let Some(date_field) = &group {
                        if let Some(prop_rc) = property_by_name.get(date_field) {
                            date_property = Some(Rc::clone(prop_rc));
                        } else {
                            bail!("View '{name}' references unknown date property: {date_field}")
                        }
                    }

                    KanbanViewType::Timeline { date_property }
                }
            };

            // Link display properties
            if let Some(display_str) = &display {
                if !display_str.trim().is_empty() {
                    // Split by commas if multiple properties are specified
                    let display_props: Vec<&str> = display_str
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect();

                    for prop_name in display_props {
                        if let Some(prop_rc) = property_by_name.get(prop_name) {
                            view_properties.push(Rc::clone(prop_rc));
                        } else {
                            // Skip unknown display properties instead of failing
                            println!(
                                "Warning: View '{name}' references unknown display property: '{prop_name}'",
                            );
                        }
                    }
                }
            }

            // Add all cards to the view for now (filtering will be applied later)
            let view_rc = Rc::new(RefCell::new(KanbanView {
                name: name.clone(),
                view_layout: kanban_view_type,
                filter,
                sort_type,
                sort_by: sort_by_prop,
                column_sorts: column_sorts_map,
                display: view_properties,
                cards: cards.clone(),
            }));

            views.push(Rc::clone(&view_rc));
            view_by_name.insert(name, view_rc);
        }

        // Update card->view references
        for view_rc in &views {
            let view = view_rc.borrow();

            for card_rc in &view.cards {
                let weak_view = Rc::downgrade(view_rc);
                card_rc.borrow_mut().views.push(weak_view);
            }
        }

        Ok(Kanban {
            properties,
            cards,
            views,
            property_by_name,
            card_by_title,
            card_by_id,
            view_by_name,
        })
    }

    // Get cards grouped by a property (for board view)
    pub fn get_cards_by_group(
        &self,
        view_name: &str,
    ) -> Result<HashMap<String, Vec<Rc<RefCell<KanbanCard>>>>> {
        let view_rc = self
            .view_by_name
            .get(view_name)
            .ok_or_else(|| anyhow!("View not found: {}", view_name))?;

        let view = view_rc.borrow();

        // Extract the group_by property from the view type
        let group_by_prop = match &view.view_layout {
            KanbanViewType::Board { group_by } => group_by
                .as_ref()
                .ok_or_else(|| anyhow!("Board view doesn't have a group_by property"))?,
            _ => return Err(anyhow!("View is not a Board view")),
        };

        let mut result = HashMap::new();

        // Initialize with all possible values for Select properties
        let prop = group_by_prop.borrow();
        if let KanbanPropertyType::Select { options } = &prop.property_type {
            for option in options {
                result.insert(option.clone(), Vec::new());
            }
        }

        // Group cards by property value
        for card_rc in &view.cards {
            let card = card_rc.borrow();

            // Find the property value
            let group_value = card
                .properties
                .iter()
                .find(|(prop, _)| Rc::ptr_eq(prop, group_by_prop))
                .map(|(_, value)| value.clone())
                .unwrap_or_else(|| "".to_string());

            // Add card to the appropriate group
            result
                .entry(group_value)
                .or_insert_with(Vec::new)
                .push(Rc::clone(card_rc));
        }

        // Apply sorting if specified
        for (group_name, cards) in result.iter_mut() {
            match view.sort_type {
                KanbanSortType::Alpha => {
                    cards.sort_by(|a, b| {
                        let a_title = &a.borrow().title;
                        let b_title = &b.borrow().title;
                        a_title.cmp(b_title)
                    });
                }
                KanbanSortType::ReverseAlpha => {
                    cards.sort_by(|a, b| {
                        let a_title = &a.borrow().title;
                        let b_title = &b.borrow().title;
                        b_title.cmp(a_title)
                    });
                }
                KanbanSortType::Manual => {
                    // If manual sorting is defined for this column
                    if let Some(order) = view.column_sorts.get(group_name) {
                        // Create a map of card ID to position for quick lookup
                        let position_map: HashMap<usize, usize> = order
                            .iter()
                            .enumerate()
                            .map(|(idx, &card_id)| (card_id, idx))
                            .collect();

                        // Sort cards based on the manual ordering
                        cards.sort_by(|a, b| {
                            let a_id = a.borrow().id;
                            let b_id = b.borrow().id;

                            // Get positions from the map, defaulting to max value if not found
                            let a_pos = position_map.get(&a_id).copied().unwrap_or(usize::MAX);
                            let b_pos = position_map.get(&b_id).copied().unwrap_or(usize::MAX);

                            // Compare positions
                            a_pos.cmp(&b_pos)
                        });
                    }
                }
                KanbanSortType::None => {
                    // No sorting, leave as is
                }
            }
        }

        Ok(result)
    }

    // Add a new card
    pub fn add_card(
        &mut self,
        title: String,
        description: String,
        property_values: HashMap<String, String>,
    ) -> Result<Rc<RefCell<KanbanCard>>> {
        // Validate properties
        let mut card_properties = Vec::new();

        for (prop_name, value) in &property_values {
            let prop_rc = self
                .property_by_name
                .get(prop_name)
                .ok_or_else(|| anyhow!("Unknown property: {}", prop_name))?;

            // Validate property value
            let prop = prop_rc.borrow();
            match &prop.property_type {
                KanbanPropertyType::Select { options } => {
                    if !options.contains(value) {
                        return Err(anyhow!("Invalid value for {}: {}", prop_name, value));
                    }
                }
                KanbanPropertyType::Checkbox => {
                    if value != "true" && value != "false" {
                        return Err(anyhow!(
                            "Invalid value for checkbox {}: {}",
                            prop_name,
                            value
                        ));
                    }
                }
                // More validation could be added for other types
                _ => {}
            }

            card_properties.push((Rc::clone(prop_rc), value.clone()));
        }

        // Generate a new ID for the card (next available ID)
        let new_id = self.cards.iter().map(|c| c.borrow().id).max().unwrap_or(0) + 1;

        // Create the card
        let card_rc = Rc::new(RefCell::new(KanbanCard {
            id: new_id,
            title: title.clone(),
            description,
            properties: card_properties,
            views: Vec::new(),
        }));

        // Update property->card references
        for (prop_rc, _) in &card_rc.borrow().properties {
            let weak_card = Rc::downgrade(&card_rc);
            prop_rc.borrow_mut().cards.push(weak_card);
        }

        // Add to all views
        for view_rc in &self.views {
            let weak_view = Rc::downgrade(view_rc);
            card_rc.borrow_mut().views.push(weak_view);

            view_rc.borrow_mut().cards.push(Rc::clone(&card_rc));
        }

        // Add to board
        self.cards.push(Rc::clone(&card_rc));
        self.card_by_title.insert(title, Rc::clone(&card_rc));
        self.card_by_id.insert(new_id, Rc::clone(&card_rc));

        Ok(card_rc)
    }

    // Move a card to a different group (column in board view)
    pub fn move_card(
        &self,
        card_title: &str,
        view_name: &str,
        new_group_value: &str,
    ) -> Result<()> {
        let card_rc = self
            .card_by_title
            .get(card_title)
            .ok_or_else(|| anyhow!("Card not found: {card_title}"))?;

        let view_rc = self
            .view_by_name
            .get(view_name)
            .ok_or_else(|| anyhow!("View not found: {view_name}"))?;

        let view = view_rc.borrow();

        // Extract the group_by property from the view type
        let group_by_prop = match &view.view_layout {
            KanbanViewType::Board { group_by } => group_by
                .as_ref()
                .ok_or(anyhow!("Board view doesn't have a group_by property"))?,
            _ => return Err(anyhow!("View is not a Board view")),
        };

        // Validate the new group value
        let prop = group_by_prop.borrow();
        if let KanbanPropertyType::Select { options } = &prop.property_type {
            if !options.contains(&new_group_value.to_string()) {
                return Err(anyhow!("Invalid group value: {new_group_value}"));
            }
        }

        // Update the card's property value
        let mut card = card_rc.borrow_mut();

        for (prop, value) in &mut card.properties {
            if Rc::ptr_eq(prop, group_by_prop) {
                *value = new_group_value.to_string();
                return Ok(());
            }
        }

        // If we get here, the card doesn't have the group_by property yet
        drop(prop); // Release the borrow

        // Add the property
        card.properties
            .push((Rc::clone(group_by_prop), new_group_value.to_string()));

        // Update property->card reference
        let weak_card = Rc::downgrade(card_rc);
        group_by_prop.borrow_mut().cards.push(weak_card);

        Ok(())
    }

    // Move a card using ID instead of title
    pub fn move_card_by_id(
        &self,
        card_id: usize,
        view_name: &str,
        new_group_value: &str,
    ) -> Result<()> {
        let card_rc = self
            .card_by_id
            .get(&card_id)
            .ok_or_else(|| anyhow!("Card not found with ID: {card_id}"))?;

        let card_title = card_rc.borrow().title.clone();
        self.move_card(&card_title, view_name, new_group_value)
    }

    // Get all views
    pub fn get_views(&self) -> Vec<Rc<RefCell<KanbanView>>> {
        self.views.clone()
    }

    // Get all properties
    pub fn get_properties(&self) -> Vec<Rc<RefCell<KanbanProperty>>> {
        self.properties.clone()
    }

    // Convert back to a ParsedDocument
    pub fn to_parsed_document(&self) -> ParsedDocument {
        let mut parsed_properties = Vec::new();

        for prop_rc in &self.properties {
            let prop = prop_rc.borrow();

            // Convert KanbanPropertyType to ParsedPropertyType
            let property_type = match &prop.property_type {
                KanbanPropertyType::Text => ParsedPropertyType::Text,
                KanbanPropertyType::Number => ParsedPropertyType::Number,
                KanbanPropertyType::Date => ParsedPropertyType::Date,
                KanbanPropertyType::Checkbox => ParsedPropertyType::Checkbox,
                KanbanPropertyType::Select { options } => ParsedPropertyType::Select {
                    options: options.clone(),
                },
            };

            parsed_properties.push(ParsedProperty {
                name: prop.name.clone(),
                property_type,
            });
        }

        let mut parsed_cards = Vec::new();

        for card_rc in &self.cards {
            let card = card_rc.borrow();

            let mut property_values = Vec::new();
            for (prop_rc, value) in &card.properties {
                let prop = prop_rc.borrow();
                property_values.push(ParsedPropertyValue {
                    property_name: prop.name.clone(),
                    value: value.clone(),
                });
            }

            parsed_cards.push(ParsedCard {
                id: card.id,
                title: card.title.clone(),
                description: card.description.clone(),
                properties: property_values,
            });
        }

        let mut parsed_views = Vec::new();

        for view_rc in &self.views {
            let view = view_rc.borrow();

            // Extract appropriate data from the view type
            let (view_type, group, sort_by) = match &view.view_layout {
                KanbanViewType::Board { group_by } => (
                    ParsedViewType::Board,
                    group_by.as_ref().map(|p| p.borrow().name.clone()),
                    None,
                ),
                KanbanViewType::Table { sort_by } => (
                    ParsedViewType::Table,
                    None,
                    sort_by.as_ref().map(|p| p.borrow().name.clone()),
                ),
                KanbanViewType::Calendar { date_property } => (
                    ParsedViewType::Calendar,
                    date_property.as_ref().map(|p| p.borrow().name.clone()),
                    None,
                ),
                KanbanViewType::Timeline { date_property } => (
                    ParsedViewType::Timeline,
                    date_property.as_ref().map(|p| p.borrow().name.clone()),
                    None,
                ),
            };

            // Convert column_sorts from HashMap to Vec<ColumnSort>
            let column_sorts = view
                .column_sorts
                .iter()
                .map(|(column, order)| ColumnSort {
                    column: column.clone(),
                    order: order.clone(),
                })
                .collect();

            parsed_views.push(ParsedView {
                name: view.name.clone(),
                layout: view_type,
                group,
                filter: view.filter.clone(),
                sort_by: sort_by.or_else(|| view.sort_by.as_ref().map(|p| p.borrow().name.clone())),
                sort_type: view.sort_type.clone(),
                column_sorts,
                display: Some(
                    view.display
                        .iter()
                        .map(|p| p.borrow().name.clone())
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            });
        }

        ParsedDocument {
            properties: parsed_properties,
            cards: parsed_cards,
            views: parsed_views,
        }
    }
}
