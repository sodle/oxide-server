resource "aws_lb" "oxide" {
  subnets            = [aws_subnet.public_a.id, aws_subnet.public_b.id]
  load_balancer_type = "application"
  security_groups    = [aws_security_group.oxide_lb.id]
}

resource "aws_alb_target_group" "oxide" {
  target_type = "ip"
  port        = 3000
  protocol    = "HTTP"
  vpc_id      = aws_vpc.vpc.id
  health_check {
    enabled = true
    path    = "/health"
  }
}

resource "aws_alb_listener" "oxide" {
  load_balancer_arn = aws_lb.oxide.arn
  protocol          = "HTTPS"
  port              = 443
  certificate_arn   = aws_acm_certificate.lb.arn
  default_action {
    type = "forward"
    forward {
      target_group {
        arn = aws_alb_target_group.oxide.arn
      }
    }
  }
}

resource "aws_alb_listener" "oxide_http_redirect" {
  load_balancer_arn = aws_lb.oxide.arn
  protocol          = "HTTP"
  port              = 80
  default_action {
    type = "redirect"
    redirect {
      status_code = "HTTP_301"
      protocol    = "HTTPS"
      port        = 443
    }
  }
}

resource "aws_security_group" "oxide_lb" {
  name   = "Oxide LB Public"
  vpc_id = aws_vpc.vpc.id
  ingress {
    cidr_blocks = ["0.0.0.0/0"]
    from_port   = 80
    to_port     = 80
    protocol    = "TCP"
  }
  ingress {
    cidr_blocks = ["0.0.0.0/0"]
    from_port   = 443
    to_port     = 443
    protocol    = "TCP"
  }
  egress {
    cidr_blocks = [aws_vpc.vpc.cidr_block]
    from_port   = 3000
    to_port     = 3000
    protocol    = "TCP"
  }
}

resource "aws_acm_certificate" "lb" {
  domain_name       = "oxide.sjodle.com"
  validation_method = "DNS"
}
