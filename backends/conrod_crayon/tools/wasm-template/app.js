
var port = process.env.PORT || 8080;
const config = require('./webpack.config.js')
const webpack = require('webpack');
const middleware = require('webpack-dev-middleware');
const compiler = webpack(config);
const express = require('express');
const app = express();

app.use(middleware(compiler, {
  // webpack-dev-middleware options t
}));

app.listen(port, () => console.log('Example app listening on port 8080!'))