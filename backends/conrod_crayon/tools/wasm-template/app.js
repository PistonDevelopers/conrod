var express = require('express')
var app = express()

// respond with "hello world" when a GET request is made to the homepage

app.use(express.static('dist'));
var port = process.env.PORT || 1337;
app.listen(port);
