resource "aws_wafv2_web_acl" "waf" {
  name   = "oxide"
  scope  = "CLOUDFRONT"
  region = "us-east-1"

  default_action {
    allow {}
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "oxide-waf"
    sampled_requests_enabled   = false
  }

  lifecycle { ignore_changes = [rule] }
}

resource "aws_wafv2_web_acl_rule" "rate_limit" {
  name        = "rate_limit"
  priority    = 1
  web_acl_arn = aws_wafv2_web_acl.waf.arn
  region      = "us-east-1"

  statement {
    rate_based_statement {
      aggregate_key_type    = "IP"
      limit                 = 100
      evaluation_window_sec = 60 * 5
    }
  }

  action {
    block {
      custom_response {
        response_code = 429
      }
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "oxide-waf-ratelimit"
    sampled_requests_enabled   = false
  }
}

resource "aws_wafv2_web_acl_rule" "aws_defaults" {
  name        = "aws_defaults"
  priority    = 0
  web_acl_arn = aws_wafv2_web_acl.waf.arn
  region      = "us-east-1"

  override_action {
    none {}
  }

  statement {
    managed_rule_group_statement {
      name        = "AWSManagedRulesCommonRuleSet"
      vendor_name = "AWS"
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "oxide-waf-defaults"
    sampled_requests_enabled   = false
  }
}
