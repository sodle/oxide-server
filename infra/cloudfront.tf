resource "aws_acm_certificate" "cloudfront" {
  domain_name       = "oxide.sjodle.com"
  region            = "us-east-1"
  validation_method = "DNS"
}

resource "aws_s3_bucket" "cloudfront_logs" {
  bucket_prefix = "sodle-oxide-cloudfront-logs-"
}

resource "aws_s3_bucket_public_access_block" "cloudfront_logs" {
  bucket            = aws_s3_bucket.cloudfront_logs.id
  block_public_acls = false
}

resource "aws_s3_bucket_ownership_controls" "cloudfront_logs" {
  bucket = aws_s3_bucket.cloudfront_logs.id
  rule {
    object_ownership = "BucketOwnerPreferred"
  }
}

resource "aws_s3_bucket_acl" "cloudfront_logs" {
  bucket = aws_s3_bucket.cloudfront_logs.id
  access_control_policy {
    owner {
      id = data.aws_canonical_user_id.current.id
    }
    grant {
      permission = "FULL_CONTROL"
      grantee {
        type = "CanonicalUser"
        id   = "c4c1ede66af53448b93c283ce9448c4ba468c9432aa01d700d3878632f77d2d0"
      }
    }
  }

  depends_on = [aws_s3_bucket_public_access_block.cloudfront_logs, aws_s3_bucket_ownership_controls.cloudfront_logs]
}

resource "aws_cloudfront_distribution" "cloudfront" {
  enabled = true
  aliases = ["oxide.sjodle.com"]
  origin {
    domain_name = aws_lb.oxide.dns_name
    origin_id   = "lb"
    vpc_origin_config {
      vpc_origin_id = aws_cloudfront_vpc_origin.alb.id
    }
  }

  viewer_certificate {
    acm_certificate_arn = aws_acm_certificate.cloudfront.arn
    ssl_support_method  = "sni-only"
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  logging_config {
    bucket = aws_s3_bucket.cloudfront_logs.bucket_domain_name
  }

  default_cache_behavior {
    allowed_methods        = ["HEAD", "DELETE", "POST", "GET", "OPTIONS", "PUT", "PATCH"]
    cached_methods         = ["GET", "HEAD", "OPTIONS"]
    target_origin_id       = "lb"
    viewer_protocol_policy = "redirect-to-https"
    cache_policy_id        = aws_cloudfront_cache_policy.default.id
  }

  web_acl_id = aws_wafv2_web_acl.waf.arn

  custom_error_response {
    error_code         = 404
    response_code      = 404
    response_page_path = "/404"
  }

  depends_on = [aws_s3_bucket_acl.cloudfront_logs]
}

resource "aws_cloudfront_cache_policy" "default" {
  name = "default"
  parameters_in_cache_key_and_forwarded_to_origin {
    cookies_config {
      cookie_behavior = "none"
    }
    headers_config {
      header_behavior = "none"
    }
    query_strings_config {
      query_string_behavior = "none"
    }
  }
}

resource "aws_cloudfront_vpc_origin" "alb" {
  vpc_origin_endpoint_config {
    arn                    = aws_lb.oxide.arn
    http_port              = 80
    https_port             = 443
    name                   = "lb"
    origin_protocol_policy = "http-only"
    origin_ssl_protocols {
      items    = ["TLSv1.2"]
      quantity = 1
    }
  }
}
