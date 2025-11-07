# AWS Icons for Agedashi

Agedashi embeds AWS Architecture Icons directly in the binary for offline use.

## Getting AWS Icons

1. Download official AWS Architecture Icons:
   - Visit: https://aws.amazon.com/architecture/icons/
   - Download the "AWS Architecture Icons" package
   - Extract the ZIP file

2. Copy PNG files to this directory with these names:
   ```
   ec2.png          - Amazon EC2
   lambda.png       - AWS Lambda
   ecs.png          - Amazon ECS
   eks.png          - Amazon EKS
   autoscaling.png  - AWS Auto Scaling
   rds.png          - Amazon RDS
   dynamodb.png     - Amazon DynamoDB
   elasticache.png  - Amazon ElastiCache
   redshift.png     - Amazon Redshift
   elb.png          - Elastic Load Balancing
   vpc.png          - Amazon VPC
   subnet.png       - VPC Subnet
   route53.png      - Amazon Route 53
   cloudfront.png   - Amazon CloudFront
   apigateway.png   - Amazon API Gateway
   s3.png           - Amazon S3
   ebs.png          - Amazon EBS
   efs.png          - Amazon EFS
   iam.png          - AWS IAM
   kms.png          - AWS KMS
   sns.png          - Amazon SNS
   sqs.png          - Amazon SQS
   kinesis.png      - Amazon Kinesis
   ```

3. The icons should be 64x64 PNG files for best results

4. Rebuild Agedashi:
   ```bash
   cargo build --release
   ```

The icons will be embedded in the binary and work completely offline!

## Alternative: Minimal Icon Set

If you don't want to download all icons, Agedashi will fall back to colored boxes
with service names for any missing icons.

## Icon Licensing

AWS Architecture Icons are provided by Amazon Web Services under their terms.
Please review AWS's icon usage guidelines at:
https://aws.amazon.com/architecture/icons/
