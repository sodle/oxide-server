resource "aws_dynamodb_table" "url_table" {
  name         = "oxide-urls"
  billing_mode = "PAY_PER_REQUEST"

  attribute {
    name = "short_code"
    type = "S"
  }

  hash_key = "short_code"
}
