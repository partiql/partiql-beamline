# E-commerce Data Tutorial

This tutorial demonstrates how to create a realistic e-commerce data generation system using PartiQL Beamline. We'll build a complete online store simulation with customers, products, orders, and analytics data, showcasing advanced patterns like static reference data, customer relationships, and realistic business metrics.

## Tutorial Overview

By the end of this tutorial, you'll have created:
- Customer profiles and behavior patterns
- Product catalogs with realistic pricing
- Dynamic order generation with seasonal patterns
- Shopping sessions and cart abandonment
- Review and rating systems
- Complete e-commerce analytics database

## Part 1: Basic Product Catalog

Let's start with a simple product catalog using static data.

### Simple Product Catalog

Create a file called `product-catalog.ion`:

```ion
rand_processes::{
    products: static_data::{
        $data: {
            product_id: UUID,
            name: LoremIpsumTitle,
            category: Uniform::{ choices: ["Electronics", "Clothing", "Books", "Home", "Sports"] },
            price: LogNormalF64::{ location: 3.0, scale: 0.8 },  // Realistic price distribution
            in_stock: Bool::{ p: 0.92 },  // 92% of products in stock
            sku: UUID,
            description: LoremIpsum::{ min_words: 10, max_words: 50 }
        }
    }
}
```

**Test the catalog:**
```bash
partiql-beamline-cli gen data \
    --seed 100 \
    --start-auto \
    --sample-count 20 \
    --script-path product-catalog.ion \
    --output-format ion-pretty
```

**Expected Output:**
```ion
{
  seed: 100,
  start: "2024-01-01T00:00:00Z",
  data: {
    products: [
      {
        product_id: "123e4567-e89b-12d3-a456-426614174000",
        name: "Premium Quality Essential Item",
        category: "Electronics",
        price: 89.99,
        in_stock: true,
        sku: "PROD-ABC123",
        description: "Lorem ipsum dolor sit amet consectetur..."
      }
      // ... more products
    ]
  }
}
```

## Part 2: Customer Generation

Now let's add customers with realistic demographics.

### Customer Profiles

Create `customers.ion`:

```ion
rand_processes::{
    // Customer demographics
    customers: static_data::{
        $data: {
            customer_id: UUID,
            id: UUID,
            name: LoremIpsumTitle,
            
            // Realistic demographics
            age: NormalF64::{ mean: 38.0, std_dev: 12.0 },
            location: {
                country: Uniform::{ choices: ["US", "CA", "GB", "AU", "DE"] },
                state: Regex::{ pattern: "[A-Z]{2}" },
                city: LoremIpsumTitle
            },
            
            // Customer segmentation
            customer_tier: Uniform::{ 
                choices: ["bronze", "silver", "gold", "platinum"],
                // Note: Real implementation might weight toward lower tiers
            },
            
            // Account information
            registration_date: Instant,
            email_verified: Bool::{ p: 0.85 },
            marketing_consent: Bool::{ p: 0.60 },
            
            // Shopping preferences
            preferred_categories: UniformArray::{
                min_size: 1,
                max_size: 3,
                element_type: Uniform::{ choices: ["Electronics", "Clothing", "Books", "Home", "Sports"] }
            }
        }
    }
}
```

**Generate customer data:**
```bash
partiql-beamline-cli gen data \
    --seed 200 \
    --start-auto \
    --sample-count 50 \
    --script-path customers.ion \
    --output-format ion-pretty
```

## Part 3: Basic Order Generation

Let's create dynamic order data that references our customers and products.

### Simple Order System

Create `simple-orders.ion`:

```ion
rand_processes::{
    $n_customers: UniformU8::{ low: 20, high: 100 },
    $n_products: UniformU8::{ low: 50, high: 200 },
    
    // Generate customer and product ID pools
    $customer_ids: $n_customers::[UUID::()],
    $product_ids: $n_products::[UUID::()],
    
    // Static customer data
    customers: static_data::{
        $data: {
            customer_id: Uniform::{ choices: $customer_ids },
            name: LoremIpsumTitle,
            email: Format::{ pattern: "customer{UUID}@shop.com" },
            registration_date: Instant
        }
    },
    
    // Static product catalog
    products: static_data::{
        $data: {
            product_id: Uniform::{ choices: $product_ids },
            name: LoremIpsumTitle,
            price: LogNormalF64::{ location: 3.2, scale: 0.9 },  // $10-500 typical range
            category: Uniform::{ choices: ["Electronics", "Clothing", "Books", "Home"] },
            in_stock: Bool::{ p: 0.90 }
        }
    },
    
    // Dynamic order events
    orders: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: minutes::20 },
        $data: {
            order_id: UUID,
            customer_id: Uniform::{ choices: $customer_ids },  // Reference existing customer
            timestamp: Instant,
            
            // Order items (1-5 items per order)
            items: UniformArray::{
                min_size: 1,
                max_size: 5,
                element_type: {
                    product_id: Uniform::{ choices: $product_ids },  // Reference existing product
                    quantity: UniformU8::{ low: 1, high: 3 },
                    unit_price: LogNormalF64::{ location: 3.0, scale: 0.7 }
                }
            },
            
            // Order status and fulfillment
            status: Uniform::{ 
                choices: ["pending", "confirmed", "shipped", "delivered", "returned"],
                // In real usage, you might weight toward "delivered"
            },
            shipping_address: {
                street: Format::{ pattern: "{UniformU16::{ low: 1, high: 9999 }} Main St" },
                city: LoremIpsumTitle,
                country: Uniform::{ choices: ["US", "CA", "GB"] },
                postal_code: Regex::{ pattern: "[0-9]{5}" }
            }
        }
    }
}
```

**Test order generation:**
```bash
partiql-beamline-cli gen data \
    --seed 300 \
    --start-auto \
    --sample-count 30 \
    --script-path simple-orders.ion \
    --dataset orders \
    --output-format text
```

## Part 4: Advanced E-commerce Patterns

### Shopping Sessions and Cart Abandonment

Create `shopping-sessions.ion`:

```ion
rand_processes::{
    $n_customers: UniformU8::{ low: 10, high: 50 },
    $customer_ids: $n_customers::[UUID::()],
    
    // Customer behavior patterns
    customers: $n_customers::[
        {
            $customer_id: $customer_ids[$@n],  // Use specific customer ID
            $session_frequency: UniformU8::{ low: 2, high: 30 },  // Days between sessions
            $purchase_probability: UniformF64::{ low: 0.15, high: 0.85 },  // Conversion rate
            
            // Customer profiles
            'customer_{$@n}': static_data::{
                $data: {
                    customer_id: $customer_id,
                    name: LoremIpsumTitle,
                    email: Format::{ pattern: "customer{$@n}@email.com" },
                    customer_segment: Uniform::{ choices: ["high_value", "regular", "bargain_hunter"] },
                    avg_order_value: LogNormalF64::{ location: 3.5, scale: 0.6 }
                }
            },
            
            // Shopping sessions for this customer
            'sessions_{$@n}': rand_process::{
                $arrival: HomogeneousPoisson::{ interarrival: days::$session_frequency },
                $data: {
                    session_id: UUID,
                    customer_id: $customer_id,
                    start_time: Instant,
                    
                    // Session characteristics
                    device_type: Uniform::{ choices: ["desktop", "mobile", "tablet"] },
                    browser: Uniform::{ choices: ["chrome", "safari", "firefox", "edge"] },
                    traffic_source: Uniform::{ choices: ["organic", "paid", "social", "email", "direct"] },
                    
                    // Browsing behavior
                    pages_viewed: UniformU8::{ low: 1, high: 20 },
                    time_on_site_minutes: LogNormalF64::{ location: 2.0, scale: 0.8 },
                    
                    // Cart behavior
                    added_to_cart: Bool::{ p: 0.4 },  // 40% add items to cart
                    purchased: Bool::{ p: $purchase_probability },  // Customer-specific conversion
                    cart_abandonment_reason: Uniform::{ 
                        choices: ["high_shipping", "price_check", "distracted", "payment_issue", "none"],
                    }
                }
            },
            
            // Actual orders from successful sessions
            'orders_{$@n}': rand_process::{
                $interval: UniformU8::{ low: 7, high: 45 },
        $arrival: HomogeneousPoisson::{ interarrival: days::$interval },
                $data: {
                    order_id: UUID,
                    customer_id: $customer_id,
                    session_id: UUID,  // Would link to session in real scenario
                    order_time: Instant,
                    
                    // Order details
                    item_count: UniformU8::{ low: 1, high: 6 },
                    subtotal: LogNormalF64::{ location: 4.0, scale: 0.7 },
                    tax_amount: UniformDecimal::{ low: 0.00, high: 50.00 },
                    shipping_cost: Uniform::{ choices: [0.00, 5.99, 9.99, 15.99, 29.99] },
                    discount_applied: UniformDecimal::{ low: 0.00, high: 25.00 },
                    
                    // Payment and fulfillment
                    payment_method: Uniform::{ choices: ["credit_card", "debit_card", "paypal", "apple_pay"] },
                    shipping_method: Uniform::{ choices: ["standard", "express", "overnight"] },
                    order_status: Uniform::{ choices: ["confirmed", "processing", "shipped", "delivered"] }
                }
            }
        }
    ]
}
```

**Generate shopping behavior data:**
```bash
partiql-beamline-cli gen data \
    --seed 400 \
    --start-iso "2024-01-01T09:00:00Z" \
    --sample-count 100 \
    --script-path shopping-sessions.ion \
    --output-format ion-pretty
```

## Part 5: Product Reviews and Ratings

Add a review system to make our e-commerce data more complete.

### Review Generation System

Create `product-reviews.ion`:

```ion
rand_processes::{
    $n_products: UniformU8::{ low: 20, high: 100 },
    $n_customers: UniformU8::{ low: 30, high: 150 },
    
    $product_ids: $n_products::[UUID::()],
    $customer_ids: $n_customers::[UUID::()],
    
    // Product catalog (static)
    products: static_data::{
        $data: {
            product_id: Uniform::{ choices: $product_ids },
            name: LoremIpsumTitle,
            category: Uniform::{ choices: ["Electronics", "Books", "Clothing", "Home"] },
            base_price: LogNormalF64::{ location: 3.5, scale: 0.8 },
            average_rating: UniformF64::{ low: 3.2, high: 4.8 }  // Most products rated well
        }
    },
    
    // Customer reviews (dynamic, less frequent than orders)
    reviews: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: hours::12 },
        $data: {
            review_id: UUID,
            product_id: Uniform::{ choices: $product_ids },
            customer_id: Uniform::{ choices: $customer_ids },
            timestamp: Instant,
            
            // Rating following realistic distribution (skewed toward higher ratings)
            rating: Uniform::{ 
                choices: [1, 2, 3, 4, 5],
                // Real e-commerce: many 4-5 star ratings, few 1-2 star
            },
            
            // Review content
            title: LoremIpsumTitle,
            review_text: LoremIpsum::{ 
                min_words: 10, 
                max_words: 100,
                optional: 0.2  // 20% ratings without text
            },
            
            // Review metadata
            verified_purchase: Bool::{ p: 0.78 },  // 78% are verified purchases
            helpful_votes: UniformU16::{ low: 0, high: 50 },
            reported: Bool::{ p: 0.02 },  // 2% of reviews get reported
            
            // Customer effort indicators
            images_attached: Bool::{ p: 0.15 },  // 15% include photos
            video_attached: Bool::{ p: 0.03 }    // 3% include videos
        }
    },
    
    // Review responses from sellers (less frequent)
    review_responses: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: days::7 },
        $data: {
            response_id: UUID,
            review_id: UUID,  // Would link to actual review
            responder_type: Uniform::{ choices: ["seller", "brand", "customer_service"] },
            response_text: LoremIpsum::{ min_words: 5, max_words: 30 },
            timestamp: Instant
        }
    }
}
```

**Test review system:**
```bash
partiql-beamline-cli gen data \
    --seed 500 \
    --start-auto \
    --sample-count 50 \
    --script-path product-reviews.ion \
    --dataset reviews \
    --output-format text
```

## Part 6: Complete E-commerce System

Now let's build a comprehensive e-commerce system with all components.

### Full E-commerce Platform

Create `ecommerce-platform.ion`:

```ion
rand_processes::{
    // === CONFIGURATION ===
    $n_customers: UniformU8::{ low: 50, high: 200 },
    $n_products: UniformU8::{ low: 100, high: 500 },
    
    // Generate ID pools for referential integrity
    $customer_ids: $n_customers::[UUID::()],
    $product_ids: $n_products::[UUID::()],
    
    // === STATIC REFERENCE DATA ===
    
    // Customer master data
    customers: static_data::{
        $data: {
            customer_id: Uniform::{ choices: $customer_ids },
            profile: {
                email: Format::{ pattern: "customer{UUID}@email.com" },
                first_name: LoremIpsumTitle,
                last_name: LoremIpsumTitle,
                phone: Regex::{ pattern: "[0-9]{3}-[0-9]{3}-[0-9]{4}", optional: 0.3 },
                
                // Demographics
                age: NormalF64::{ mean: 35.0, std_dev: 15.0 },
                gender: Uniform::{ choices: ["M", "F", "Other"], optional: 0.1 },
                
                // Location
                address: {
                    street: Format::{ pattern: "{UniformU16::{ low: 1, high: 9999 }} {LoremIpsumTitle} St" },
                    city: LoremIpsumTitle,
                    state: Regex::{ pattern: "[A-Z]{2}" },
                    zip: Regex::{ pattern: "[0-9]{5}" },
                    country: Uniform::{ choices: ["US", "CA", "GB", "AU"] }
                }
            },
            
            // Account settings
            account: {
                registration_date: Instant,
                tier: Uniform::{ choices: ["standard", "premium", "vip"] },
                email_verified: Bool::{ p: 0.88 },
                phone_verified: Bool::{ p: 0.65 },
                marketing_opt_in: Bool::{ p: 0.45 }
            }
        }
    },
    
    // Product catalog
    products: static_data::{
        $data: {
            product_id: Uniform::{ choices: $product_ids },
            info: {
                name: LoremIpsumTitle,
                brand: LoremIpsumTitle,
                category: Uniform::{ choices: ["Electronics", "Clothing", "Books", "Home", "Sports", "Beauty"] },
                subcategory: LoremIpsumTitle,
                description: LoremIpsum::{ min_words: 15, max_words: 80 }
            },
            
            // Pricing and inventory
            pricing: {
                base_price: LogNormalF64::{ location: 3.4, scale: 0.9 },
                sale_price: LogNormalF64::{ location: 3.2, scale: 0.8, optional: 0.7 },  // 30% on sale
                currency: "USD"
            },
            
            inventory: {
                in_stock: Bool::{ p: 0.92 },
                stock_quantity: UniformU16::{ low: 0, high: 1000 },
                warehouse_location: Uniform::{ choices: ["US-WEST", "US-EAST", "EU", "ASIA"] }
            },
            
            // Product metrics
            metrics: {
                view_count: LogNormalF64::{ location: 6.0, scale: 1.2 },
                rating_average: UniformF64::{ low: 3.0, high: 5.0 },
                rating_count: LogNormalF64::{ location: 3.0, scale: 1.5 },
                conversion_rate: UniformF64::{ low: 0.02, high: 0.15 }
            }
        }
    },
    
    // === DYNAMIC TRANSACTIONAL DATA ===
    
    // Order events with realistic timing
    orders: rand_process::{
        $interval: UniformU8::{ low: 3, high: 45 },
        $arrival: HomogeneousPoisson::{ interarrival: minutes::$interval },
        $data: {
            order_id: UUID,
            customer_id: Uniform::{ choices: $customer_ids },
            order_date: Instant,
            
            // Order composition
            items: UniformArray::{
                min_size: 1,
                max_size: 8,
                element_type: {
                    product_id: Uniform::{ choices: $product_ids },
                    quantity: UniformU8::{ low: 1, high: 4 },
                    unit_price: LogNormalF64::{ location: 3.3, scale: 0.8 },
                    discount_percent: UniformF64::{ low: 0.0, high: 25.0, optional: 0.6 }
                }
            },
            
            // Order totals
            financials: {
                subtotal: LogNormalF64::{ location: 4.2, scale: 0.9 },
                tax_amount: UniformDecimal::{ low: 0.00, high: 75.00 },
                shipping_cost: Uniform::{ choices: [0.00, 4.99, 9.99, 19.99] },
                total_amount: LogNormalF64::{ location: 4.3, scale: 0.9 }
            },
            
            // Fulfillment
            fulfillment: {
                shipping_method: Uniform::{ choices: ["standard", "expedited", "overnight", "pickup"] },
                estimated_delivery: Instant,  // Would be calculated from order_date
                tracking_number: Regex::{ pattern: "[A-Z]{2}[0-9]{12}" },
                status: Uniform::{ choices: ["processing", "shipped", "delivered", "exception"] }
            },
            
            // Payment
            payment: {
                method: Uniform::{ choices: ["credit_card", "debit_card", "paypal", "apple_pay", "google_pay"] },
                processor: Uniform::{ choices: ["stripe", "paypal", "square"] },
                transaction_id: UUID,
                authorized: Bool::{ p: 0.96 },  // 96% successful authorization
                captured: Bool::{ p: 0.98 }    // 98% successful capture
            }
        }
    },
    
    // Product reviews (less frequent than orders)
    reviews: rand_process::{
        $interval: UniformU8::{ low: 6, high: 72 },
        $arrival: HomogeneousPoisson::{ interarrival: hours::$interval },
        $data: {
            review_id: UUID,
            product_id: Uniform::{ choices: $product_ids },
            customer_id: Uniform::{ choices: $customer_ids },
            order_id: UUID,  // Would link to actual order
            timestamp: Instant,
            
            // Rating distribution (realistic - skewed toward positive)
            rating: Uniform::{ 
                choices: [5, 5, 5, 4, 4, 3, 2, 1],  // Weighted toward higher ratings
            },
            
            content: {
                title: LoremIpsumTitle,
                text: LoremIpsum::{ 
                    min_words: 8, 
                    max_words: 150,
                    optional: 0.15  // 15% ratings without review text
                },
                images_count: UniformU8::{ low: 0, high: 5 },
                video_attached: Bool::{ p: 0.05 }
            },
            
            // Review metadata
            verified_purchase: Bool::{ p: 0.82 },
            helpful_count: UniformU16::{ low: 0, high: 100 },
            reported_count: UniformU8::{ low: 0, high: 5 },
            seller_responded: Bool::{ p: 0.35 }
        }
    },
    
    // Customer support tickets
    support_tickets: rand_process::{
        $interval: UniformU16::{ low: 4, high: 96 },
        $arrival: HomogeneousPoisson::{ interarrival: hours::$interval },
        $data: {
            ticket_id: UUID,
            customer_id: Uniform::{ choices: $customer_ids },
            order_id: UUID,  // Would link to actual order causing issue
            created_at: Instant,
            
            // Ticket classification
            category: Uniform::{ choices: [
                "order_issue", "product_question", "shipping_delay", 
                "return_request", "payment_issue", "account_problem"
            ]},
            priority: Uniform::{ choices: ["low", "medium", "high", "urgent"] },
            
            // Ticket content
            subject: LoremIpsumTitle,
            description: LoremIpsum::{ min_words: 20, max_words: 200 },
            
            // Resolution
            status: Uniform::{ choices: ["open", "in_progress", "resolved", "closed"] },
            resolution_time_hours: LogNormalF64::{ location: 2.5, scale: 1.0, optional: 0.3 },
            satisfaction_rating: UniformU8::{ low: 1, high: 5, optional: 0.4 }
        }
    }
}
```

**Generate complete e-commerce data:**
```bash
partiql-beamline-cli gen data \
    --seed 600 \
    --start-iso "2024-01-01T00:00:00Z" \
    --sample-count 200 \
    --script-path ecommerce-platform.ion \
    --output-format ion-pretty
```

## Part 7: Seasonal and Promotional Patterns

### Holiday Shopping Simulation

Create `seasonal-patterns.ion`:

```ion
rand_processes::{
    // Black Friday / Cyber Monday pattern
    holiday_shopping: rand_process::{
        // Much higher frequency during sales events
        $interval: UniformU16::{ low: 30, high: 300 },
        $arrival: HomogeneousPoisson::{ interarrival: seconds::$interval },
        $data: {
            order_id: UUID,
            customer_id: UUID,
            timestamp: Instant,
            
            // Holiday shopping characteristics
            order_source: Uniform::{ choices: ["mobile_app", "website", "email_campaign"] },
            promotion_applied: Bool::{ p: 0.85 },  // 85% use promotions during sales
            
            // Higher order values during sales
            order_total: LogNormalF64::{ location: 4.8, scale: 0.6 },  // Higher AOV
            item_count: UniformU8::{ low: 2, high: 12 },  // More items per order
            
            // Gift orders
            is_gift: Bool::{ p: 0.4 },  // 40% are gifts during holidays
            gift_message: LoremIpsum::{ 
                min_words: 5, 
                max_words: 25,
                optional: 0.7  // Most gifts don't include messages
            },
            
            // Shipping preferences during busy season
            shipping_urgency: Uniform::{ choices: ["standard", "expedited", "overnight", "express"] },
            delivery_instructions: LoremIpsum::{ 
                min_words: 3, 
                max_words: 15,
                optional: 0.8
            }
        }
    },
    
    // Regular shopping pattern (baseline)
    regular_shopping: rand_process::{
        $interval: UniformU8::{ low: 1, high: 12 },
        $arrival: HomogeneousPoisson::{ interarrival: hours::$interval },
        $data: {
            order_id: UUID,
            customer_id: UUID,
            timestamp: Instant,
            
            // Regular shopping characteristics
            order_source: Uniform::{ choices: ["website", "mobile_app", "search"] },
            promotion_applied: Bool::{ p: 0.25 },  // 25% use promotions normally
            
            // Normal order patterns
            order_total: LogNormalF64::{ location: 3.8, scale: 0.9 },
            item_count: UniformU8::{ low: 1, high: 4 },
            
            is_gift: Bool::{ p: 0.05 },  // 5% are gifts normally
            shipping_urgency: Uniform::{ choices: ["standard", "standard", "expedited"] }  // Mostly standard
        }
    }
}
```

## Part 8: Analytics and Business Intelligence

### E-commerce Analytics Data

Create `ecommerce-analytics.ion`:

```ion
rand_processes::{
    // Website analytics events
    web_analytics: rand_process::{
        $interval: UniformU8::{ low: 1, high: 30 },
        $arrival: HomogeneousPoisson::{ interarrival: seconds::$interval },
        $data: {
            event_id: UUID,
            session_id: UUID,
            customer_id: UUID,  // May be null for anonymous users
            timestamp: Instant,
            
            // Event classification
            event_type: Uniform::{ choices: [
                "page_view", "product_view", "add_to_cart", "remove_from_cart",
                "search", "filter", "sort", "checkout_start", "purchase", "exit"
            ]},
            
            // Page/product context
            page_url: Format::{ pattern: "/products/{UUID}" },
            page_title: LoremIpsumTitle,
            product_id: UUID,  // Would be null for non-product pages
            category: Uniform::{ choices: ["Electronics", "Clothing", "Books", "Home"] },
            
            // Technical details
            user_agent: Uniform::{ choices: ["Chrome/91.0", "Safari/14.0", "Firefox/89.0", "Edge/91.0"] },
            device: {
                type: Uniform::{ choices: ["desktop", "mobile", "tablet"] },
                os: Uniform::{ choices: ["Windows", "macOS", "iOS", "Android", "Linux"] },
                screen_width: Uniform::{ choices: [1920, 1366, 375, 768, 1440] },
                screen_height: Uniform::{ choices: [1080, 768, 667, 1024, 900] }
            },
            
            // Engagement metrics
            time_on_page_seconds: LogNormalF64::{ location: 3.5, scale: 1.0 },
            scroll_percentage: UniformF64::{ low: 0.1, high: 1.0 },
            clicks_on_page: UniformU8::{ low: 0, high: 15 }
        }
    },
    
    // Business metrics aggregates (daily summaries)
    daily_metrics: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: days::1 },
        $data: {
            metric_date: Instant,
            
            // Sales metrics
            sales: {
                total_revenue: LogNormalF64::{ location: 8.5, scale: 0.8 },
                order_count: LogNormalF64::{ location: 4.0, scale: 0.6 },
                average_order_value: LogNormalF64::{ location: 3.9, scale: 0.5 },
                units_sold: LogNormalF64::{ location: 5.0, scale: 0.7 }
            },
            
            // Traffic metrics
            traffic: {
                unique_visitors: LogNormalF64::{ location: 6.5, scale: 0.8 },
                page_views: LogNormalF64::{ location: 7.2, scale: 0.9 },
                session_count: LogNormalF64::{ location: 6.8, scale: 0.8 },
                bounce_rate: UniformF64::{ low: 0.2, high: 0.8 }
            },
            
            // Conversion metrics
            conversion: {
                conversion_rate: UniformF64::{ low: 0.02, high: 0.08 },
                cart_abandonment_rate: UniformF64::{ low: 0.65, high: 0.85 },
                email_signup_rate: UniformF64::{ low: 0.05, high: 0.20 }
            }
        }
    }
}
```

**Generate analytics data:**
```bash
partiql-beamline-cli gen data \
    --seed 700 \
    --start-iso "2024-01-01T00:00:00Z" \
    --sample-count 30 \
    --script-path ecommerce-analytics.ion \
    --output-format ion-pretty
```

## Part 9: Query Generation for E-commerce

Generate analytical queries for your e-commerce data:

```bash
# Generate business intelligence queries
partiql-beamline-cli query basic \
    --seed 800 \
    --start-auto \
    --script-path ecommerce-platform.ion \
    --sample-count 10 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 4 \
        --tbl-flt-path-depth-max 3 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-all
```

**Example generated queries:**
```sql
SELECT * FROM orders AS orders 
WHERE (orders.financials.total_amount > 100.0 
       AND orders.fulfillment.status = 'delivered')

SELECT * FROM reviews AS reviews 
WHERE (reviews.rating >= 4 AND reviews.verified_purchase = true)

SELECT * FROM customers AS customers 
WHERE (customers.account.tier = 'premium' 
       AND customers.profile.age BETWEEN 25 AND 45)

SELECT * FROM products AS products 
WHERE (products.pricing.base_price < 50.0 
       AND products.inventory.in_stock = true)
```

## Part 10: Database Generation

Create a complete e-commerce database:

```bash
# Generate comprehensive e-commerce database
partiql-beamline-cli gen db beamline-lite \
    --seed 900 \
    --start-iso "2024-01-01T00:00:00Z" \
    --script-path ecommerce-platform.ion \
    --sample-count 10000 \
    --catalog-name ecommerce-demo-db \
    --catalog-path ./ecommerce-database

# Check generated database structure
tree ecommerce-database/ecommerce-demo-db/

# Examine some schemas
cat ecommerce-database/ecommerce-demo-db/orders.shape.sql
cat ecommerce-database/ecommerce-demo-db/customers.shape.sql
```

## Part 11: Advanced Analytics Patterns

### Customer Lifetime Value Simulation

Create `customer-analytics.ion`:

```ion
rand_processes::{
    $n_customers: UniformU8::{ low: 100, high: 500 },
    
    customer_analytics: $n_customers::[
        {
            $customer_id: UUID::(),
            $lifetime_value: LogNormalF64::{ location: 6.0, scale: 1.2 }::(),  // $400-2000 CLV
            $churn_probability: UniformF64::{ low: 0.05, high: 0.3 }::(),
            
            // Customer profile with CLV characteristics
            'customer_profile_{$@n}': static_data::{
                $data: {
                    customer_id: $customer_id,
                    segment: Uniform::{ choices: ["high_value", "medium_value", "low_value", "at_risk"] },
                    predicted_lifetime_value: $lifetime_value,
                    churn_score: $churn_probability,
                    
                    // Behavioral indicators
                    first_purchase_date: Instant,
                    last_purchase_days_ago: UniformU16::{ low: 0, high: 365 },
                    total_orders: LogNormalF64::{ location: 2.5, scale: 1.0 },
                    total_spent: $lifetime_value,
                    avg_order_frequency_days: UniformU8::{ low: 7, high: 90 }
                }
            },
            
            // Purchase events for this customer
            'customer_purchases_{$@n}': rand_process::{
                $frequency: UniformU8::{ low: 3, high: 45 },  // Days between purchases
                $arrival: HomogeneousPoisson::{ interarrival: days::$frequency },
                $data: {
                    order_id: UUID,
                    customer_id: $customer_id,
                    purchase_date: Instant,
                    
                    // Order characteristics based on customer value
                    order_value: LogNormalF64::{ 
                        location: 3.0, 
                        scale: 0.8 
                    },
                    
                    // Loyalty indicators
                    loyalty_points_earned: UniformU16::{ low: 10, high: 500 },
                    coupon_used: Bool::{ p: 0.3 },
                    referral_made: Bool::{ p: 0.1 }
                }
            }
        }
    ]
}
```

## Key E-commerce Concepts Demonstrated

Through this tutorial, you've learned:

1. **Static Reference Data**: Product catalogs, customer profiles
2. **Dynamic Transactional Data**: Orders, reviews, support tickets
3. **Realistic Business Patterns**: Seasonal shopping, cart abandonment, conversion rates
4. **Customer Segmentation**: Different customer types with varying behaviors
5. **Referential Integrity**: Linking customers, products, and orders
6. **Business Analytics**: Web analytics, daily metrics, customer lifetime value
7. **Complex Data Structures**: Nested objects for rich business data
8. **Statistical Realism**: Using appropriate distributions for business metrics
9. **Multi-Dataset Systems**: Complete business data ecosystem
10. **Query Generation**: Business intelligence and analytical queries

## Next Steps and Extensions

Try these advanced e-commerce patterns:

1. **Inventory Management**: Track stock levels, reorder points, suppliers
2. **Marketing Campaigns**: Email campaigns, conversion tracking, A/B testing
3. **Fraud Detection**: Suspicious orders, payment failures, account security
4. **Return and Refund Processing**: Return reasons, refund amounts, restocking
5. **Personalization Data**: Recommendation engines, browsing history, preferences
6. **Multi-Channel Sales**: Marketplace integration, social commerce, mobile apps
7. **Supply Chain**: Vendor relationships, procurement, logistics
8. **International Markets**: Currency conversion, localization, tax compliance

This tutorial provides a comprehensive foundation for modeling any e-commerce or retail system with PartiQL Beamline. The patterns can be adapted for B2B commerce, marketplaces, subscription services, and other commercial applications.
