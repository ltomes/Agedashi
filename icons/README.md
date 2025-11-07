# AWS Icons for Agedashi

Agedashi automatically downloads and caches AWS Architecture Icons on first run.

## Automatic Icon Download (Default)

On first run, Agedashi automatically:
1. Downloads AWS service icons from the diagrams library (MIT licensed)
2. Caches them to `~/.cache/agedashi/icons/`
3. Uses cached icons for all future runs (works offline after first download)

No manual setup required! Just run the tool and icons will be downloaded automatically.

## Manual Icon Setup (Optional)

If you prefer to use official AWS icons or customize the icon set:

### Option 1: Official AWS Architecture Icons

1. Download official AWS Architecture Icons:
   - Visit: https://aws.amazon.com/architecture/icons/
   - Download the "AWS Architecture Icons" package (ZIP file)
   - Extract the ZIP file

2. Find the PNG files for services you use (usually in folders like "Resource-Icons_*/Res_*")

3. Copy PNG files to this directory with these names:
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

### Option 2: Alternative Icon Sources

You can also use icons from:
- IcePanelIO: https://icon.icepanel.io/AWS/ (SVG format - convert to PNG)
- Any other AWS icon set (ensure they're square PNG files)

### Icon Format Requirements

For best results, icons should be:
- **Format**: PNG with transparent background
- **Size**: 64x64 pixels or larger (square aspect ratio)
- **Quality**: Clear, high-contrast icons work best
- **Background**: Transparent PNG recommended

### Converting SVG to PNG

If you have SVG icons, convert them to PNG:
```bash
# Using ImageMagick
convert icon.svg -resize 64x64 -background none icon.png

# Using Inkscape
inkscape icon.svg --export-type=png --export-width=64 --export-height=64 -o icon.png
```

### Building with Icons

After adding PNG files to this directory:
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
