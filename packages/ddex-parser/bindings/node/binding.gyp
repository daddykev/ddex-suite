// bindings/node/binding.gyp
{
  "targets": [
    {
      "target_name": "ddex_parser",
      "sources": [],
      "conditions": [
        ["OS=='mac'", {
          "libraries": [
            "../target/release/libddex_parser_node.dylib"
          ]
        }],
        ["OS=='linux'", {
          "libraries": [
            "../target/release/libddex_parser_node.so"
          ]
        }],
        ["OS=='win'", {
          "libraries": [
            "../target/release/ddex_parser_node.dll"
          ]
        }]
      ]
    }
  ]
}