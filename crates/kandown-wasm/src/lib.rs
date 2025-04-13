// crates/kandown-wasm/src/lib.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use reflect_to::Reflect;

#[wasm_bindgen]
pub struct WasmKanbanBoard {
    board: kandown::Kanban,
    // Store mapping from ID to title for efficient lookups
    id_to_title: HashMap<String, String>,
}

// Comprehensive data structure for board views
#[derive(Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct KanbanViewData {
    views: Vec<ViewInfo>,
    columns: Vec<ColumnData>,
    cards: HashMap<String, CardData>,
    properties: Vec<PropertyData>,
    group_by_property: Option<String>,
    items: HashMap<String, Vec<String>>, // For DnD structure
}

#[derive(Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
struct ViewInfo {
    name: String,
    layout: String,
    group_by: Option<String>,
    sort_by: Option<String>,
    sort_type: String,
}

#[derive(Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
struct PropertyData {
    name: String,
    property_type: String,
    options: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
struct ColumnData {
    id: String,
    title: String,
    cards: Vec<String>, // Just the card IDs
}

#[derive(Clone, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "camelCase")]
struct CardData {
    id: String,
    title: String,
    description: String,
    properties: HashMap<String, String>,
}

#[wasm_bindgen]
impl WasmKanbanBoard {
    #[wasm_bindgen(constructor)]
    pub fn new(markdown: &str) -> Result<WasmKanbanBoard, JsValue> {
        let parsed_doc = kandown::ParsedDocument::try_from(markdown)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let board = kandown::Kanban::from_document(parsed_doc)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(WasmKanbanBoard {
            board,
            id_to_title: HashMap::new(),
        })
    }

    // Get all available views
    #[wasm_bindgen(js_name = getViewNames)]
    pub fn get_view_names(&self) -> Result<js_sys::Array, JsValue> {
        let views = self.board.get_views();
        let result = js_sys::Array::new();

        for view in views.iter() {
            let v = view.borrow();
            result.push(&JsValue::from_str(&v.name));
        }

        Ok(result)
    }

    // Get comprehensive data for a specific view
    #[wasm_bindgen(js_name = getViewData)]
    pub fn get_view_data(&mut self, view_name: &str) -> Result<JsValue, JsValue> {
        let grouped_cards = self
            .board
            .get_cards_by_group(view_name)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let mut columns = Vec::new();
        let mut id_to_title = HashMap::new();
        let mut cards_map = HashMap::new();
        let mut items_map = HashMap::new();

        // Process columns and cards
        for (group_name, cards) in &grouped_cards {
            let mut column_cards = Vec::new();

            for card_rc in cards {
                let card = card_rc.borrow();
                let mut properties = HashMap::new();

                for (prop_rc, value) in &card.properties {
                    let prop = prop_rc.borrow();
                    properties.insert(prop.name.clone(), value.clone());
                }

                // Generate a unique ID based on the card
                let card_id = format!("card_{}", card.id);

                // Store the card data
                let card_data = CardData {
                    id: card_id.clone(),
                    title: card.title.clone(),
                    description: card.description.clone(),
                    properties,
                };

                // Update maps
                id_to_title.insert(card_id.clone(), card.title.clone());
                cards_map.insert(card_id.clone(), card_data);
                column_cards.push(card_id.clone());
            }

            columns.push(ColumnData {
                id: group_name.clone(),
                title: group_name.clone(),
                cards: column_cards.clone(),
            });

            // Also build the items map for DnD
            items_map.insert(group_name.clone(), column_cards);
        }

        // Get properties information
        let properties = self
            .board
            .get_properties()
            .iter()
            .map(|prop_rc| {
                let prop = prop_rc.borrow();
                let (prop_type, options) = match &prop.property_type {
                    kandown::KanbanPropertyType::Text => ("Text", None),
                    kandown::KanbanPropertyType::Number => ("Number", None),
                    kandown::KanbanPropertyType::Date => ("Date", None),
                    kandown::KanbanPropertyType::Checkbox => ("Checkbox", None),
                    kandown::KanbanPropertyType::Select { options } => {
                        ("Select", Some(options.clone()))
                    }
                };

                PropertyData {
                    name: prop.name.clone(),
                    property_type: prop_type.to_string(),
                    options,
                }
            })
            .collect();

        // Update instance id mapping
        self.id_to_title = id_to_title;

        // Get the current view info to determine group by property
        let view_rc = self
            .board
            .view_by_name
            .get(view_name)
            .ok_or_else(|| JsValue::from_str(&format!("View not found: {}", view_name)))?;

        let view = view_rc.borrow();
        let group_by_property = match &view.view_layout {
            kandown::KanbanViewType::Board { group_by } => {
                group_by.as_ref().map(|p| p.borrow().name.clone())
            }
            _ => None,
        };

        // Create full view data
        let view_data = KanbanViewData {
            views: self
                .board
                .get_views()
                .iter()
                .map(|view_rc| {
                    let view = view_rc.borrow();
                    let (layout, group_by, sort_by) = match &view.view_layout {
                        kandown::KanbanViewType::Board { group_by } => (
                            "Board",
                            group_by.as_ref().map(|p| p.borrow().name.clone()),
                            None,
                        ),
                        kandown::KanbanViewType::Table { sort_by } => (
                            "Table",
                            None,
                            sort_by.as_ref().map(|p| p.borrow().name.clone()),
                        ),
                        kandown::KanbanViewType::Calendar { date_property } => (
                            "Calendar",
                            date_property.as_ref().map(|p| p.borrow().name.clone()),
                            None,
                        ),
                        kandown::KanbanViewType::Timeline { date_property } => (
                            "Timeline",
                            date_property.as_ref().map(|p| p.borrow().name.clone()),
                            None,
                        ),
                    };

                    ViewInfo {
                        name: view.name.clone(),
                        layout: layout.to_string(),
                        group_by,
                        sort_by: sort_by
                            .or_else(|| view.sort_by.as_ref().map(|p| p.borrow().name.clone())),
                        sort_type: format!("{:?}", view.sort_type),
                    }
                })
                .collect(),
            columns,
            cards: cards_map,
            properties,
            group_by_property,
            items: items_map,
        };

        let json =
            serde_json::to_string(&view_data).map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(JsValue::from_str(&json))
    }

    // Move a card between columns
    #[wasm_bindgen(js_name = moveCard)]
    pub fn move_card(
        &self,
        card_id: &str,
        _source_column: &str,
        destination_column: &str,
        view_name: &str,
    ) -> Result<(), JsValue> {
        // Extract the numeric ID from the card_id string (e.g., "card_1" -> 1)
        let id_parts: Vec<&str> = card_id.split('_').collect();
        if id_parts.len() != 2 || id_parts[0] != "card" {
            return Err(JsValue::from_str(&format!(
                "Invalid card ID format: {}",
                card_id
            )));
        }

        let numeric_id = id_parts[1]
            .parse::<usize>()
            .map_err(|_| JsValue::from_str(&format!("Invalid card ID number: {}", id_parts[1])))?;

        // Move the card using the numeric ID
        self.board
            .move_card_by_id(numeric_id, view_name, destination_column)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // Add a new card
    #[wasm_bindgen(js_name = addCard)]
    pub fn add_card(
        &mut self,
        title: &str,
        description: &str,
        properties_json: &str,
    ) -> Result<String, JsValue> {
        let properties: HashMap<String, String> =
            serde_json::from_str(properties_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

        let card_rc = self
            .board
            .add_card(title.to_string(), description.to_string(), properties)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Use the card's ID for consistent identification
        let card_id = format!("card_{}", card_rc.borrow().id);

        // Update the mapping
        self.id_to_title.insert(card_id.clone(), title.to_string());

        Ok(card_id)
    }

    // Get the current markdown representation of the board
    #[wasm_bindgen(js_name = getMarkdown)]
    pub fn get_markdown(&self) -> String {
        self.board.to_parsed_document().to_string()
    }
}
