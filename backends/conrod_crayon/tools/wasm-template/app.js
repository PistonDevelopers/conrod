var express = require('express')
var app = express()

// respond with "hello world" when a GET request is made to the homepage

app.use(express.static('dist'));
app.listen(3000);
