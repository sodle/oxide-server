terraform {
  required_version = ">= 1.0.0" # Ensure that the Terraform version is 1.0.0 or higher

  required_providers {
    aws = {
      source  = "hashicorp/aws" # Specify the source of the AWS provider
      version = "~> 4.0"        # Use a version of the AWS provider that is compatible with version
    }
  }
}

provider "aws" {
  region = "us-west-2"

  default_tags {
    tags = {
      project = "oxide"
    }
  }
}

resource "aws_dynamodb_table" "url_table" {
  name         = "oxide-urls"
  billing_mode = "PAY_PER_REQUEST"

  attribute {
    name = "short_code"
    type = "S"
  }

  hash_key = "short_code"
}
