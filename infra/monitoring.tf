data "aws_iam_policy_document" "grafana_cloudwatch_role_trustee" {
  statement {
    principals {
      identifiers = ["008923505280"]
      type        = "AWS"
    }
    actions = ["sts:AssumeRole"]
    condition {
      test     = "StringEquals"
      values   = ["1567418"]
      variable = "sts:ExternalId"
    }
  }
}

data "aws_iam_policy_document" "grafana_cloudwatch_role" {
  statement {
    actions = [
      "cloudwatch:Describe*", "cloudwatch:Get*", "cloudwatch:List*",
      "logs:Describe*", "logs:Get*", "logs:List*",
      "athena:Get*", "athena:List*", "athena:StartQueryExecution",
      "glue:Get*", "glue:List*", "glue:Describe*",
    ]
    resources = ["*"]
  }
  statement {
    actions   = ["s3:GetObject", "s3:ListBucket"]
    resources = ["arn:aws:s3:::sodle-cost-usage-report", "arn:aws:s3:::sodle-cost-usage-report/*"]
  }
}

resource "aws_iam_policy" "grafana_cloudwatch_role" {
  policy = data.aws_iam_policy_document.grafana_cloudwatch_role.json
  name   = "oxide_grafana_cloudwatch"
}

resource "aws_iam_role" "grafana_cloudwatch_role" {
  assume_role_policy = data.aws_iam_policy_document.grafana_cloudwatch_role_trustee.json
  name               = "oxide_grafana_cloudwatch"
}

resource "aws_iam_role_policy_attachment" "grafana_cloudwatch_role" {
  policy_arn = aws_iam_policy.grafana_cloudwatch_role.arn
  role       = aws_iam_role.grafana_cloudwatch_role.name
}

output "grafana_cloudwatch_role_arn" {
  value = aws_iam_role.grafana_cloudwatch_role.arn
}
