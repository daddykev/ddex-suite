//! Type generation for TypeScript and Python from JSON Schema

use super::*;
use indexmap::IndexSet;

impl SchemaGenerator {
    /// Convert JSON Schema to TypeScript interface definition
    pub(crate) fn schema_to_typescript(&self, name: &str, schema: &JsonSchema) -> Result<String, BuildError> {
        let mut output = String::new();
        let mut imports = IndexSet::new();
        
        // Add JSDoc comment if description exists
        if let Some(ref description) = schema.description {
            output.push_str(&format!("/**\n * {}\n */\n", description));
        }
        
        match schema.schema_type.as_deref() {
            Some("object") => {
                output.push_str(&format!("export interface {} {{\n", name));
                
                if let Some(ref properties) = schema.properties {
                    let required = schema.required.as_ref().map(|r| r.iter().collect::<IndexSet<_>>()).unwrap_or_default();
                    
                    for (prop_name, prop_schema) in properties {
                        let optional = if required.contains(prop_name) { "" } else { "?" };
                        let ts_type = self.schema_to_typescript_type(prop_schema, &mut imports)?;
                        
                        // Add property documentation
                        if let Some(ref description) = prop_schema.description {
                            output.push_str(&format!("  /** {} */\n", description));
                        }
                        
                        output.push_str(&format!("  {}{}: {};\n", prop_name, optional, ts_type));
                    }
                }
                
                output.push_str("}\n");
            },
            Some("array") => {
                if let Some(ref items) = schema.items {
                    let item_type = self.schema_to_typescript_type(items, &mut imports)?;
                    output.push_str(&format!("export type {} = {}[];\n", name, item_type));
                } else {
                    output.push_str(&format!("export type {} = any[];\n", name));
                }
            },
            _ => {
                if let Some(ref enum_values) = schema.enum_values {
                    let enum_variants: Vec<String> = enum_values.iter()
                        .map(|v| match v {
                            JsonValue::String(s) => format!("\"{}\"", s),
                            JsonValue::Number(n) => n.to_string(),
                            JsonValue::Bool(b) => b.to_string(),
                            _ => "unknown".to_string(),
                        })
                        .collect();
                    
                    output.push_str(&format!("export type {} = {};\n", name, enum_variants.join(" | ")));
                } else {
                    let ts_type = self.schema_to_typescript_type(schema, &mut imports)?;
                    output.push_str(&format!("export type {} = {};\n", name, ts_type));
                }
            }
        }
        
        // Add imports at the beginning if needed
        if !imports.is_empty() {
            let import_statements: Vec<String> = imports.into_iter()
                .map(|import| format!("import {{ {} }} from './types';", import))
                .collect();
            output = format!("{}\n\n{}", import_statements.join("\n"), output);
        }
        
        Ok(output)
    }
    
    /// Convert JSON Schema to TypeScript type string
    fn schema_to_typescript_type(&self, schema: &JsonSchema, imports: &mut IndexSet<String>) -> Result<String, BuildError> {
        // Handle references
        if let Some(ref reference) = schema.reference {
            if reference.starts_with("#/$defs/") {
                let type_name = &reference[8..];
                imports.insert(type_name.to_string());
                return Ok(type_name.to_string());
            }
        }
        
        // Handle type unions (anyOf, oneOf)
        if let Some(ref any_of) = schema.any_of {
            let union_types: Result<Vec<String>, BuildError> = any_of.iter()
                .map(|s| self.schema_to_typescript_type(s, imports))
                .collect();
            return Ok(format!("({})", union_types?.join(" | ")));
        }
        
        if let Some(ref one_of) = schema.one_of {
            let union_types: Result<Vec<String>, BuildError> = one_of.iter()
                .map(|s| self.schema_to_typescript_type(s, imports))
                .collect();
            return Ok(format!("({})", union_types?.join(" | ")));
        }
        
        // Handle enum values
        if let Some(ref enum_values) = schema.enum_values {
            let variants: Vec<String> = enum_values.iter()
                .map(|v| match v {
                    JsonValue::String(s) => format!("\"{}\"", s),
                    JsonValue::Number(n) => n.to_string(),
                    JsonValue::Bool(b) => b.to_string(),
                    _ => "unknown".to_string(),
                })
                .collect();
            return Ok(variants.join(" | "));
        }
        
        // Handle basic types
        match schema.schema_type.as_deref() {
            Some("string") => {
                // Check for specific string formats
                match schema.format.as_deref() {
                    Some("date") => Ok("string /* date: YYYY-MM-DD */".to_string()),
                    Some("date-time") => Ok("string /* date-time: ISO 8601 */".to_string()),
                    Some("uri") => Ok("string /* URI */".to_string()),
                    _ => {
                        // Add pattern information if available
                        if let Some(ref pattern) = schema.pattern {
                            Ok(format!("string /* pattern: {} */", pattern))
                        } else {
                            Ok("string".to_string())
                        }
                    }
                }
            },
            Some("number") => Ok("number".to_string()),
            Some("integer") => Ok("number".to_string()),
            Some("boolean") => Ok("boolean".to_string()),
            Some("null") => Ok("null".to_string()),
            Some("array") => {
                if let Some(ref items) = schema.items {
                    let item_type = self.schema_to_typescript_type(items, imports)?;
                    Ok(format!("{}[]", item_type))
                } else {
                    Ok("any[]".to_string())
                }
            },
            Some("object") => {
                if let Some(ref properties) = schema.properties {
                    let mut object_type = String::from("{\n");
                    let required = schema.required.as_ref().map(|r| r.iter().collect::<IndexSet<_>>()).unwrap_or_default();
                    
                    for (prop_name, prop_schema) in properties {
                        let optional = if required.contains(prop_name) { "" } else { "?" };
                        let prop_type = self.schema_to_typescript_type(prop_schema, imports)?;
                        object_type.push_str(&format!("    {}{}: {};\n", prop_name, optional, prop_type));
                    }
                    
                    object_type.push_str("  }");
                    Ok(object_type)
                } else {
                    Ok("Record<string, any>".to_string())
                }
            },
            _ => Ok("any".to_string()),
        }
    }
    
    /// Convert JSON Schema to Python TypedDict definition
    pub(crate) fn schema_to_python(&self, name: &str, schema: &JsonSchema) -> Result<String, BuildError> {
        let mut output = String::new();
        let mut imports = IndexSet::new();
        
        // Add docstring if description exists
        if let Some(ref description) = schema.description {
            output.push_str(&format!("\"\"\"{}.\"\"\"\n", description));
        }
        
        match schema.schema_type.as_deref() {
            Some("object") => {
                // Determine if we need total=False for optional fields
                let required = schema.required.as_ref().map(|r| r.iter().collect::<IndexSet<_>>()).unwrap_or_default();
                let has_optional = schema.properties.as_ref()
                    .map(|props| props.keys().any(|k| !required.contains(k)))
                    .unwrap_or(false);
                
                let total_param = if has_optional { ", total=False" } else { "" };
                output.push_str(&format!("class {}(TypedDict{}):\n", name, total_param));
                
                if let Some(ref properties) = schema.properties {
                    for (prop_name, prop_schema) in properties {
                        let python_type = self.schema_to_python_type(prop_schema, &mut imports)?;
                        let field_type = if required.contains(prop_name) || !has_optional {
                            python_type
                        } else {
                            format!("Optional[{}]", python_type)
                        };
                        
                        // Add field documentation
                        if let Some(ref description) = prop_schema.description {
                            output.push_str(&format!("    # {}\n", description));
                        }
                        
                        output.push_str(&format!("    {}: {}\n", prop_name, field_type));
                    }
                    
                    if properties.is_empty() {
                        output.push_str("    pass\n");
                    }
                } else {
                    output.push_str("    pass\n");
                }
            },
            Some("array") => {
                if let Some(ref items) = schema.items {
                    let item_type = self.schema_to_python_type(items, &mut imports)?;
                    output.push_str(&format!("{} = List[{}]\n", name, item_type));
                } else {
                    output.push_str(&format!("{} = List[Any]\n", name));
                }
            },
            _ => {
                if let Some(ref enum_values) = schema.enum_values {
                    // Create a Literal type for enum values
                    let enum_variants: Vec<String> = enum_values.iter()
                        .map(|v| match v {
                            JsonValue::String(s) => format!("\"{}\"", s),
                            JsonValue::Number(n) => n.to_string(),
                            JsonValue::Bool(b) => b.to_string(),
                            _ => "None".to_string(),
                        })
                        .collect();
                    
                    output.push_str(&format!("{} = Literal[{}]\n", name, enum_variants.join(", ")));
                } else {
                    let python_type = self.schema_to_python_type(schema, &mut imports)?;
                    output.push_str(&format!("{} = {}\n", name, python_type));
                }
            }
        }
        
        Ok(output)
    }
    
    /// Convert JSON Schema to Python type string
    fn schema_to_python_type(&self, schema: &JsonSchema, imports: &mut IndexSet<String>) -> Result<String, BuildError> {
        // Handle references
        if let Some(ref reference) = schema.reference {
            if reference.starts_with("#/$defs/") {
                let type_name = &reference[8..];
                return Ok(type_name.to_string());
            }
        }
        
        // Handle type unions
        if let Some(ref any_of) = schema.any_of {
            let union_types: Result<Vec<String>, BuildError> = any_of.iter()
                .map(|s| self.schema_to_python_type(s, imports))
                .collect();
            return Ok(format!("Union[{}]", union_types?.join(", ")));
        }
        
        if let Some(ref one_of) = schema.one_of {
            let union_types: Result<Vec<String>, BuildError> = one_of.iter()
                .map(|s| self.schema_to_python_type(s, imports))
                .collect();
            return Ok(format!("Union[{}]", union_types?.join(", ")));
        }
        
        // Handle enum values
        if let Some(ref enum_values) = schema.enum_values {
            let variants: Vec<String> = enum_values.iter()
                .map(|v| match v {
                    JsonValue::String(s) => format!("\"{}\"", s),
                    JsonValue::Number(n) => n.to_string(),
                    JsonValue::Bool(b) => b.to_string(),
                    _ => "None".to_string(),
                })
                .collect();
            return Ok(format!("Literal[{}]", variants.join(", ")));
        }
        
        // Handle basic types
        match schema.schema_type.as_deref() {
            Some("string") => {
                match schema.format.as_deref() {
                    Some("date") => Ok("str  # date: YYYY-MM-DD".to_string()),
                    Some("date-time") => Ok("datetime  # ISO 8601 datetime".to_string()),
                    Some("uri") => Ok("str  # URI".to_string()),
                    _ => Ok("str".to_string()),
                }
            },
            Some("number") => Ok("float".to_string()),
            Some("integer") => Ok("int".to_string()),
            Some("boolean") => Ok("bool".to_string()),
            Some("null") => Ok("None".to_string()),
            Some("array") => {
                if let Some(ref items) = schema.items {
                    let item_type = self.schema_to_python_type(items, imports)?;
                    Ok(format!("List[{}]", item_type))
                } else {
                    Ok("List[Any]".to_string())
                }
            },
            Some("object") => {
                if schema.properties.is_some() {
                    // For inline objects, use Dict with type hints if possible
                    Ok("Dict[str, Any]".to_string())
                } else {
                    Ok("Dict[str, Any]".to_string())
                }
            },
            _ => Ok("Any".to_string()),
        }
    }
    
    /// Generate OpenAPI specification from schema
    pub fn generate_openapi_spec(&self, schema: &JsonSchema) -> Result<String, BuildError> {
        let mut openapi = serde_json::json!({
            "openapi": "3.0.3",
            "info": {
                "title": format!("DDEX Builder API - ERN {}", self.version_string()),
                "description": format!("REST API for DDEX Builder with {} profile", self.profile_string()),
                "version": "1.0.0"
            },
            "servers": [{
                "url": "https://api.ddex-builder.example.com",
                "description": "DDEX Builder API Server"
            }],
            "paths": {
                "/build": {
                    "post": {
                        "summary": "Build DDEX message",
                        "description": "Create a DDEX XML message from structured data",
                        "requestBody": {
                            "required": true,
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/BuildRequest"
                                    }
                                }
                            }
                        },
                        "responses": {
                            "200": {
                                "description": "Successfully generated DDEX XML",
                                "content": {
                                    "application/xml": {
                                        "schema": {
                                            "type": "string"
                                        }
                                    }
                                }
                            },
                            "400": {
                                "description": "Invalid request data",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "$ref": "#/components/schemas/Error"
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "/validate": {
                    "post": {
                        "summary": "Validate DDEX data",
                        "description": "Validate structured data against DDEX schema",
                        "requestBody": {
                            "required": true,
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/BuildRequest"
                                    }
                                }
                            }
                        },
                        "responses": {
                            "200": {
                                "description": "Validation result",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "$ref": "#/components/schemas/ValidationResult"
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "/schema": {
                    "get": {
                        "summary": "Get JSON Schema",
                        "description": "Retrieve the current JSON Schema for validation",
                        "parameters": [{
                            "name": "version",
                            "in": "query",
                            "description": "DDEX version",
                            "schema": {
                                "type": "string",
                                "enum": ["4.1", "4.2", "4.3"]
                            }
                        }, {
                            "name": "profile",
                            "in": "query", 
                            "description": "Message profile",
                            "schema": {
                                "type": "string",
                                "enum": ["AudioAlbum", "AudioSingle", "VideoAlbum", "VideoSingle", "Mixed"]
                            }
                        }],
                        "responses": {
                            "200": {
                                "description": "JSON Schema",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "type": "object"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "components": {
                "schemas": {}
            }
        });
        
        // Add schema definitions to components
        if let Some(ref definitions) = schema.definitions {
            let mut components_schemas = serde_json::Map::new();
            
            for (name, def_schema) in definitions {
                // Convert our JsonSchema to OpenAPI schema format
                let openapi_schema = self.convert_to_openapi_schema(def_schema)?;
                components_schemas.insert(name.clone(), openapi_schema);
            }
            
            // Add standard error schema
            components_schemas.insert("Error".to_string(), json!({
                "type": "object",
                "required": ["code", "message"],
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Error code"
                    },
                    "message": {
                        "type": "string", 
                        "description": "Error message"
                    },
                    "field": {
                        "type": "string",
                        "description": "Field that caused the error"
                    }
                }
            }));
            
            // Add validation result schema
            components_schemas.insert("ValidationResult".to_string(), json!({
                "type": "object",
                "required": ["valid", "errors", "warnings"],
                "properties": {
                    "valid": {
                        "type": "boolean",
                        "description": "Whether validation passed"
                    },
                    "errors": {
                        "type": "array",
                        "items": {"$ref": "#/components/schemas/Error"},
                        "description": "Validation errors"
                    },
                    "warnings": {
                        "type": "array",
                        "items": {"$ref": "#/components/schemas/Error"},
                        "description": "Validation warnings"
                    }
                }
            }));
            
            openapi["components"]["schemas"] = JsonValue::Object(components_schemas);
        }
        
        serde_json::to_string_pretty(&openapi)
            .map_err(|e| BuildError::InvalidFormat {
                field: "openapi".to_string(),
                message: format!("Failed to serialize OpenAPI spec: {}", e),
            })
    }
    
    /// Convert our JsonSchema format to OpenAPI 3.0 schema format
    fn convert_to_openapi_schema(&self, schema: &JsonSchema) -> Result<JsonValue, BuildError> {
        let mut openapi_schema = serde_json::Map::new();
        
        if let Some(ref title) = schema.title {
            openapi_schema.insert("title".to_string(), JsonValue::String(title.clone()));
        }
        
        if let Some(ref description) = schema.description {
            openapi_schema.insert("description".to_string(), JsonValue::String(description.clone()));
        }
        
        if let Some(ref schema_type) = schema.schema_type {
            openapi_schema.insert("type".to_string(), JsonValue::String(schema_type.clone()));
        }
        
        if let Some(ref properties) = schema.properties {
            let mut openapi_properties = serde_json::Map::new();
            for (name, prop_schema) in properties {
                openapi_properties.insert(name.clone(), self.convert_to_openapi_schema(prop_schema)?);
            }
            openapi_schema.insert("properties".to_string(), JsonValue::Object(openapi_properties));
        }
        
        if let Some(ref required) = schema.required {
            let required_array: Vec<JsonValue> = required.iter()
                .map(|r| JsonValue::String(r.clone()))
                .collect();
            openapi_schema.insert("required".to_string(), JsonValue::Array(required_array));
        }
        
        if let Some(additional_properties) = schema.additional_properties {
            openapi_schema.insert("additionalProperties".to_string(), JsonValue::Bool(additional_properties));
        }
        
        if let Some(ref items) = schema.items {
            openapi_schema.insert("items".to_string(), self.convert_to_openapi_schema(items)?);
        }
        
        if let Some(ref enum_values) = schema.enum_values {
            openapi_schema.insert("enum".to_string(), JsonValue::Array(enum_values.clone()));
        }
        
        if let Some(ref pattern) = schema.pattern {
            openapi_schema.insert("pattern".to_string(), JsonValue::String(pattern.clone()));
        }
        
        if let Some(ref format) = schema.format {
            openapi_schema.insert("format".to_string(), JsonValue::String(format.clone()));
        }
        
        if let Some(min_length) = schema.min_length {
            openapi_schema.insert("minLength".to_string(), JsonValue::Number(min_length.into()));
        }
        
        if let Some(max_length) = schema.max_length {
            openapi_schema.insert("maxLength".to_string(), JsonValue::Number(max_length.into()));
        }
        
        if let Some(ref examples) = schema.examples {
            openapi_schema.insert("examples".to_string(), JsonValue::Array(examples.clone()));
        }
        
        if let Some(ref reference) = schema.reference {
            openapi_schema.insert("$ref".to_string(), JsonValue::String(reference.clone()));
        }
        
        Ok(JsonValue::Object(openapi_schema))
    }
}