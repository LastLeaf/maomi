const path = require("path");

module.exports = {
  mode: "production",
  entry: "./bootstrap",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "main.js",
  },
};
