# Financial Transactions Tutorial

This tutorial shows how to create realistic financial data using PartiQL Beamline. We'll build a comprehensive financial system simulation including bank accounts, transactions, market data, and compliance reporting, demonstrating how statistical distributions can model real-world financial patterns.

## Tutorial Overview

By the end of this tutorial, you'll have created:
- Bank account and customer data
- Realistic transaction patterns
- Market data and trading systems  
- Risk assessment and fraud detection data
- Regulatory compliance reporting
- Complete financial analytics database

## Part 1: Basic Account and Transactions

Let's start with simple bank accounts and basic transactions.

### Simple Banking System

Create a file called `basic-banking.ion`:

```ion
rand_processes::{
    // Static account data
    accounts: static_data::{
        $data: {
            account_id: UUID,
            account_number: Regex::{ pattern: "[0-9]{10}" },
            account_type: Uniform::{ choices: ["checking", "savings", "credit", "investment"] },
            customer_id: UUID,
            
            // Account details
            opening_date: Date,
            status: Uniform::{ choices: ["active", "closed", "frozen", "dormant"] },
            currency: Uniform::{ choices: ["USD", "EUR", "GBP", "CAD"] },
            
            // Initial balance using log-normal (realistic for account balances)
            balance: LogNormalF64::{ location: 7.5, scale: 1.2 },  // ~$2K median, wide range
            
            // Account limits and features
            daily_limit: LogNormalF64::{ location: 6.9, scale: 0.8 },  // ~$1K daily limit
            overdraft_limit: UniformDecimal::{ low: 0.00, high: 2500.00, optional: 0.6 },
            interest_rate: UniformF64::{ low: 0.001, high: 0.05 }  // 0.1% to 5% APR
        }
    },
    
    // Dynamic transaction data
    transactions: rand_process::{
        $r: UniformU8::{ low: 2, high: 48 },
        $arrival: HomogeneousPoisson::{ interarrival: hours::$r },
        $data: {
            transaction_id: UUID,
            account_id: UUID,  // Would link to actual account
            timestamp: Instant,
            
            // Transaction details
            amount: LogNormalF64::{ location: 4.0, scale: 1.5 },  // Wide range of amounts
            transaction_type: Uniform::{ choices: [
                "deposit", "withdrawal", "transfer", "payment", 
                "purchase", "fee", "interest", "refund"
            ]},
            
            // Transaction context
            description: LoremIpsum::{ min_words: 3, max_words: 15 },
            merchant: LoremIpsumTitle,
            category: Uniform::{ choices: [
                "groceries", "gas", "restaurant", "retail", "utilities", 
                "healthcare", "entertainment", "travel", "other"
            ]},
            
            // Status and processing
            status: Uniform::{ choices: ["pending", "completed", "failed", "reversed"] },
            processing_time_seconds: LogNormalF64::{ location: 2.0, scale: 1.0 },
            
            // Security and compliance
            risk_score: UniformF64::{ low: 0.0, high: 1.0 },
            flagged_for_review: Bool::{ p: 0.02 },  // 2% flagged
            location: Regex::{ pattern: "[A-Z]{2}", optional: 0.3 }
        }
    }
}
```

**Test basic banking:**
```bash
partiql-beamline-cli gen data \
    --seed 100 \
    --start-iso "2024-01-01T00:00:00Z" \
    --sample-count 50 \
    --script-path basic-banking.ion \
    --output-format ion-pretty
```

## Part 2: Customer Financial Profiles

Add realistic customer data with financial characteristics.

### Customer Financial Data

Create `financial-customers.ion`:

```ion
rand_processes::{
    customers: static_data::{
        $data: {
            customer_id: UUID,
            
            // Personal information
            personal: {
                first_name: LoremIpsumTitle,
                last_name: LoremIpsumTitle,
                ssn: Regex::{ pattern: "[0-9]{3}-[0-9]{2}-[0-9]{4}" },  // Format only
                date_of_birth: Date,
                phone: Regex::{ pattern: "[0-9]{3}-[0-9]{3}-[0-9]{4}" },
                email: Format::{ pattern: "customer{UUID}@email.com" }
            },
            
            // Address information
            address: {
                street: Format::{ pattern: "{UniformU16::{ low: 1, high: 9999 }} {LoremIpsumTitle} St" },
                city: LoremIpsumTitle,
                state: Regex::{ pattern: "[A-Z]{2}" },
                zip: Regex::{ pattern: "[0-9]{5}" },
                country: Uniform::{ choices: ["US", "CA", "GB", "AU"] }
            },
            
            // Financial profile
            financial_profile: {
                // Income distribution (log-normal is realistic)
                annual_income: LogNormalF64::{ location: 10.8, scale: 0.8 },  // ~$50K median
                
                // Credit score (normal distribution around 700)
                credit_score: NormalF64::{ mean: 700.0, std_dev: 120.0 },
                
                // Employment status
                employment_status: Uniform::{ choices: ["employed", "self_employed", "unemployed", "retired", "student"] },
                employer: LoremIpsumTitle,
                
                // Banking relationship
                customer_since: Date,
                risk_category: Uniform::{ choices: ["low", "medium", "high"] },
                kyc_verified: Bool::{ p: 0.95 },  // 95% KYC verified
                
                // Financial behavior indicators
                avg_monthly_balance: LogNormalF64::{ location: 7.0, scale: 1.0 },
                transaction_volume_monthly: LogNormalF64::{ location: 4.5, scale: 0.8 },
                overdraft_history: Bool::{ p: 0.15 },  // 15% have overdraft history
                loan_defaults: Bool::{ p: 0.05 }       // 5% have defaults
            }
        }
    }
}
```

**Test customer profiles:**
```bash
partiql-beamline-cli gen data \
    --seed 200 \
    --start-auto \
    --sample-count 25 \
    --script-path financial-customers.ion \
    --output-format ion-pretty
```

## Part 3: Advanced Transaction Patterns

Model different types of financial transactions with realistic patterns.

### Comprehensive Transaction System

Create `financial-transactions.ion`:

```ion
rand_processes::{
    $n_accounts: UniformU8::{ low: 50, high: 200 },
    $account_ids: $n_accounts::[UUID::()],
    
    // Account master data
    accounts: static_data::{
        $data: {
            account_id: Uniform::{ choices: $account_ids },
            customer_id: UUID,
            account_type: Uniform::{ choices: ["checking", "savings", "credit", "loan", "investment"] },
            
            // Account characteristics affect transaction patterns
            risk_profile: Uniform::{ choices: ["conservative", "moderate", "aggressive"] },
            account_tier: Uniform::{ choices: ["basic", "premium", "private"] },
            
            // Financial limits
            daily_transaction_limit: LogNormalF64::{ location: 8.0, scale: 0.8 },
            monthly_transaction_limit: LogNormalF64::{ location: 10.0, scale: 0.6 },
            current_balance: LogNormalF64::{ location: 7.2, scale: 1.4 }
        }
    },
    
    // Regular transactions - most common
    regular_transactions: rand_process::{
        $r: UniformU8::{ low: 6, high: 72 },
        $arrival: HomogeneousPoisson::{ interarrival: hours::$r },
        $data: {
            transaction_id: UUID,
            account_id: Uniform::{ choices: $account_ids },
            timestamp: Instant,
            
            // Transaction characteristics
            amount: LogNormalF64::{ location: 3.5, scale: 1.2 },  // $10-1000 typical range
            transaction_type: Uniform::{ choices: [
                "debit_card", "credit_card", "ach_debit", "ach_credit", 
                "wire_transfer", "check", "atm_withdrawal", "mobile_payment"
            ]},
            
            // Merchant and location
            merchant: {
                name: LoremIpsumTitle,
                category: Uniform::{ choices: [
                    "grocery", "gas_station", "restaurant", "retail", "pharmacy",
                    "utilities", "insurance", "mortgage", "subscription", "other"
                ]},
                mcc_code: UniformU16::{ low: 1000, high: 9999 }  // Merchant Category Code
            },
            
            // Geographic and security information
            location: {
                city: LoremIpsumTitle,
                state: Regex::{ pattern: "[A-Z]{2}" },
                country: Uniform::{ choices: ["US", "CA", "MX"] },
                zip_code: Regex::{ pattern: "[0-9]{5}" }
            },
            
            // Transaction metadata
            channel: Uniform::{ choices: ["online", "pos", "atm", "mobile", "phone", "branch"] },
            authorization_code: Regex::{ pattern: "[A-Z0-9]{6}" },
            
            // Risk and compliance
            risk_score: WeibullF64::{ shape: 0.8, scale: 0.3 },  // Most low risk, few high risk
            aml_flagged: Bool::{ p: 0.001 },  // 0.1% flagged for AML
            requires_review: Bool::{ p: 0.005 }  // 0.5% require manual review
        }
    },
    
    // High-value transactions - less frequent, more scrutiny
    high_value_transactions: rand_process::{
        $r: UniformU8::{ low: 1, high: 30 },
        $arrival: HomogeneousPoisson::{ interarrival: days::$r },
        $data: {
            transaction_id: UUID,
            account_id: Uniform::{ choices: $account_ids },
            timestamp: Instant,
            
            // High-value characteristics
            amount: LogNormalF64::{ location: 8.0, scale: 0.8 },  // $1K-100K+ range
            transaction_type: Uniform::{ choices: [
                "wire_transfer", "large_check", "investment_transfer", 
                "real_estate", "business_payment", "loan_disbursement"
            ]},
            
            // Enhanced documentation for large amounts
            purpose: LoremIpsum::{ min_words: 5, max_words: 25 },
            source_of_funds: Uniform::{ choices: [
                "salary", "investment_sale", "loan_proceeds", "business_revenue", 
                "inheritance", "gift", "real_estate_sale", "other"
            ]},
            
            // Compliance requirements
            compliance: {
                ctr_filed: Bool::{ p: 0.1 },    // Currency Transaction Report
                sar_filed: Bool::{ p: 0.02 },   // Suspicious Activity Report  
                ofac_checked: Bool::{ p: 1.0 }, // Always check OFAC for high-value
                enhanced_due_diligence: Bool::{ p: 0.15 },
                approval_required: Bool::{ p: 0.8 },
                approved_by: UUID,
                approval_timestamp: Instant
            },
            
            // Enhanced risk assessment
            risk_factors: {
                cross_border: Bool::{ p: 0.2 },
                high_risk_country: Bool::{ p: 0.05 },
                politically_exposed_person: Bool::{ p: 0.01 },
                unusual_pattern: Bool::{ p: 0.1 },
                risk_score: UniformF64::{ low: 0.3, high: 1.0 }  // Higher baseline risk
            }
        }
    },
    
    // Recurring transactions (automated payments)
    recurring_transactions: rand_process::{
        $r: UniformU8::{ low: 7, high: 31 },
        $arrival: HomogeneousPoisson::{ interarrival: days::$r },
        $data: {
            transaction_id: UUID,
            account_id: Uniform::{ choices: $account_ids },
            timestamp: Instant,
            
            // Recurring payment characteristics
            amount: LogNormalF64::{ location: 5.0, scale: 0.6 },  // $50-500 typical
            payment_type: Uniform::{ choices: [
                "mortgage", "rent", "auto_loan", "student_loan", "credit_card",
                "utilities", "insurance", "subscription", "savings_transfer"
            ]},
            
            // Recurring payment metadata
            payee: LoremIpsumTitle,
            schedule: Uniform::{ choices: ["weekly", "biweekly", "monthly", "quarterly"] },
            autopay: Bool::{ p: 0.8 },  // 80% are autopay
            
            // Status (most succeed, occasional failures)
            status: Uniform::{ choices: ["completed", "completed", "completed", "failed"] },
            failure_reason: Uniform::{ 
                choices: ["insufficient_funds", "account_closed", "technical_error", null],
                // Only relevant when status = "failed"
            }
        }
    }
}
```

**Test basic financial system:**
```bash
partiql-beamline-cli gen data \
    --seed 300 \
    --start-iso "2024-01-01T00:00:00Z" \
    --sample-count 40 \
    --script-path basic-banking.ion \
    --dataset transactions \
    --output-format text
```

## Part 4: Market Data and Trading

Financial systems often include market data and trading activities.

### Trading and Market Data

Create `trading-system.ion`:

```ion
rand_processes::{
    // Static securities reference data
    securities: static_data::{
        $data: {
            symbol: Regex::{ pattern: "[A-Z]{3,4}" },
            company_name: LoremIpsumTitle,
            sector: Uniform::{ choices: [
                "Technology", "Healthcare", "Financial", "Consumer", 
                "Industrial", "Energy", "Utilities", "Real Estate"
            ]},
            exchange: Uniform::{ choices: ["NYSE", "NASDAQ", "AMEX"] },
            
            // Security characteristics
            market_cap: LogNormalF64::{ location: 9.0, scale: 2.0 },  // $1B-1T range
            shares_outstanding: LogNormalF64::{ location: 8.5, scale: 1.0 },
            avg_daily_volume: LogNormalF64::{ location: 6.0, scale: 1.5 },
            
            // Current pricing (would be updated in real system)
            current_price: LogNormalF64::{ location: 3.5, scale: 0.8 },
            day_change_percent: NormalF64::{ mean: 0.0, std_dev: 2.5 },
            volatility: UniformF64::{ low: 0.1, high: 0.8 }
        }
    },
    
    // Trading activity
    trades: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::30 },
        $data: {
            trade_id: UUID,
            account_id: UUID,
            symbol: Regex::{ pattern: "[A-Z]{3,4}" },
            timestamp: Instant,
            
            // Trade details
            side: Uniform::{ choices: ["buy", "sell"] },
            quantity: LogNormalF64::{ location: 4.0, scale: 1.2 },  // Share quantities
            price_per_share: LogNormalF64::{ location: 3.2, scale: 0.9 },
            
            // Order types and execution
            order_type: Uniform::{ choices: ["market", "limit", "stop", "stop_limit"] },
            time_in_force: Uniform::{ choices: ["day", "gtc", "immediate", "fill_or_kill"] },
            
            // Execution details
            execution_price: LogNormalF64::{ location: 3.2, scale: 0.9 },
            commission: UniformDecimal::{ low: 0.00, high: 29.99 },
            fees: UniformDecimal::{ low: 0.00, high: 5.00 },
            
            // Trade status
            status: Uniform::{ choices: ["filled", "partial", "cancelled", "rejected"] },
            fill_percentage: UniformF64::{ low: 0.0, high: 1.0 },
            
            // Market data at execution
            market_conditions: {
                bid_price: LogNormalF64::{ location: 3.2, scale: 0.9 },
                ask_price: LogNormalF64::{ location: 3.2, scale: 0.9 },
                bid_size: LogNormalF64::{ location: 5.0, scale: 0.8 },
                ask_size: LogNormalF64::{ location: 5.0, scale: 0.8 },
                last_price: LogNormalF64::{ location: 3.2, scale: 0.9 },
                volume: LogNormalF64::{ location: 6.0, scale: 1.0 }
            }
        }
    },
    
    // Market data feed (high frequency)
    market_data: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: seconds::5 },
        $data: {
            symbol: Regex::{ pattern: "[A-Z]{3,4}" },
            timestamp: Instant,
            
            // OHLC data
            open: LogNormalF64::{ location: 3.5, scale: 0.8 },
            high: LogNormalF64::{ location: 3.5, scale: 0.8 },
            low: LogNormalF64::{ location: 3.5, scale: 0.8 },
            close: LogNormalF64::{ location: 3.5, scale: 0.8 },
            volume: LogNormalF64::{ location: 5.5, scale: 1.2 },
            
            // Real-time quotes
            bid: LogNormalF64::{ location: 3.5, scale: 0.8 },
            ask: LogNormalF64::{ location: 3.5, scale: 0.8 },
            bid_size: UniformU32::{ low: 100, high: 10000 },
            ask_size: UniformU32::{ low: 100, high: 10000 },
            
            // Market indicators
            change_percent: NormalF64::{ mean: 0.0, std_dev: 3.0 },  // Daily price changes
            volatility: UniformF64::{ low: 0.1, high: 0.9 },
            
            // Trading session info
            session: Uniform::{ choices: ["pre_market", "regular", "after_hours"] },
            exchange: Uniform::{ choices: ["NYSE", "NASDAQ", "AMEX"] }
        }
    }
}
```

**Generate trading data:**
```bash
partiql-beamline-cli gen data \
    --seed 400 \
    --start-auto \
    --sample-count 100 \
    --script-path trading-system.ion \
    --dataset trades \
    --output-format text
```

## Part 5: Fraud Detection and Risk Management

Model suspicious activities and risk assessment patterns.

### Fraud Detection System

Create `fraud-detection.ion`:

```ion
rand_processes::{
    $n_accounts: UniformU8::{ low: 100, high: 500 },
    $account_ids: $n_accounts::[UUID::()],
    
    // Normal transaction patterns (baseline)
    normal_transactions: rand_process::{
        $r: UniformU8::{ low: 4, high: 48 },
        $arrival: HomogeneousPoisson::{ interarrival: hours::$r },
        $data: {
            transaction_id: UUID,
            account_id: Uniform::{ choices: $account_ids },
            timestamp: Instant,
            
            // Normal transaction amounts
            amount: LogNormalF64::{ location: 3.8, scale: 0.9 },
            merchant: LoremIpsumTitle,
            location: Uniform::{ choices: ["home_city", "nearby_city"] },
            
            // Low risk characteristics
            risk_score: WeibullF64::{ shape: 0.5, scale: 0.2 },  // Most very low risk
            fraud_indicators: {
                unusual_location: Bool::{ p: 0.05 },
                unusual_amount: Bool::{ p: 0.03 },
                unusual_time: Bool::{ p: 0.02 },
                velocity_anomaly: Bool::{ p: 0.01 },
                merchant_risk: Bool::{ p: 0.02 }
            },
            
            transaction_outcome: Uniform::{ choices: ["approved", "approved", "approved", "declined"] }
        }
    },
    
    // Suspicious transactions - rare but important
    suspicious_transactions: rand_process::{
        $r: UniformU8::{ low: 5, high: 90 },
        $arrival: HomogeneousPoisson::{ interarrival: days::$r },
        $data: {
            transaction_id: UUID,
            account_id: Uniform::{ choices: $account_ids },
            timestamp: Instant,
            
            // Suspicious patterns
            amount: UniformAnyOf::{
                types: [
                    LogNormalF64::{ location: 6.0, scale: 0.5 },  // Large amounts
                    UniformDecimal::{ low: 9999.99, high: 10000.01 },  // Just under reporting threshold
                    LogNormalF64::{ location: 2.0, scale: 0.3 }   // Many small amounts (structuring)
                ]
            },
            
            // Fraud risk indicators
            risk_score: UniformF64::{ low: 0.7, high: 1.0 },  // High risk scores
            fraud_indicators: {
                unusual_location: Bool::{ p: 0.6 },      // Often unusual location
                foreign_country: Bool::{ p: 0.3 },
                high_risk_merchant: Bool::{ p: 0.4 },
                velocity_anomaly: Bool::{ p: 0.7 },      // Unusual transaction frequency
                amount_anomaly: Bool::{ p: 0.8 },        // Unusual amount patterns
                time_anomaly: Bool::{ p: 0.3 },          // Unusual timing
                device_anomaly: Bool::{ p: 0.2 }
            },
            
            // Investigation and resolution
            investigation: {
                flagged_by_system: Bool::{ p: 0.9 },
                manual_review_required: Bool::{ p: 0.8 },
                investigation_status: Uniform::{ choices: ["pending", "in_progress", "completed"] },
                fraud_confirmed: Bool::{ p: 0.3 },  // 30% of suspicious transactions are actual fraud
                false_positive: Bool::{ p: 0.7 },   // 70% are false positives
                
                // Response actions
                account_blocked: Bool::{ p: 0.1 },
                card_reissued: Bool::{ p: 0.2 },
                customer_contacted: Bool::{ p: 0.9 },
                law_enforcement_notified: Bool::{ p: 0.05 }
            },
            
            transaction_outcome: Uniform::{ choices: ["declined", "approved_with_conditions", "blocked"] }
        }
    },
    
    // Compliance reporting events
    compliance_reports: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: days::UniformU8::{ low: 1, high: 7 } },
        $data: {
            report_id: UUID,
            report_type: Uniform::{ choices: ["CTR", "SAR", "OFAC_ALERT", "BSA_REPORT"] },
            filing_date: Instant,
            
            // Report details
            account_ids: UniformArray::{
                min_size: 1,
                max_size: 5,
                element_type: Uniform::{ choices: $account_ids }
            },
            
            total_amount: LogNormalF64::{ location: 9.0, scale: 1.0 },  // Large amounts trigger reports
            
            // Regulatory information
            regulatory: {
                filing_institution: "Bank ABC",
                examiner: LoremIpsumTitle,
                priority: Uniform::{ choices: ["routine", "priority", "immediate"] },
                confidential: Bool::{ p: 0.8 },
                
                // Follow-up actions
                requires_follow_up: Bool::{ p: 0.3 },
                regulator_response: Uniform::{ 
                    choices: ["no_action", "request_info", "investigation", "enforcement"],
                    optional: 0.7
                }
            }
        }
    }
}
```

**Generate fraud detection data:**
```bash
partiql-beamline-cli gen data \
    --seed 500 \
    --start-iso "2024-01-01T00:00:00Z" \
    --sample-count 75 \
    --script-path fraud-detection.ion \
    --dataset suspicious_transactions \
    --output-format ion-pretty
```

## Part 6: Credit and Lending

Model loan origination, credit decisions, and payment patterns.

### Credit and Loan System

Create `credit-lending.ion`:

```ion
rand_processes::{
    $n_customers: UniformU8::{ low: 50, high: 200 },
    $customer_ids: $n_customers::[UUID::()],
    
    // Credit applications
    credit_applications: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: days::UniformU8::{ low: 1, high: 14 } },
        $data: {
            application_id: UUID,
            customer_id: Uniform::{ choices: $customer_ids },
            application_date: Instant,
            
            // Loan details requested
            loan_type: Uniform::{ choices: [
                "personal_loan", "auto_loan", "mortgage", "credit_card", 
                "business_loan", "student_loan", "home_equity"
            ]},
            requested_amount: LogNormalF64::{ location: 9.0, scale: 1.5 },  // $1K-$1M range
            requested_term_months: Uniform::{ choices: [12, 24, 36, 48, 60, 84, 120, 240, 360] },
            
            // Applicant information
            applicant: {
                annual_income: LogNormalF64::{ location: 10.5, scale: 0.8 },
                employment_years: UniformU8::{ low: 0, high: 40 },
                credit_score: NormalF64::{ mean: 680.0, std_dev: 100.0 },
                debt_to_income_ratio: UniformF64::{ low: 0.1, high: 0.6 },
                
                // Housing
                housing_status: Uniform::{ choices: ["own", "rent", "other"] },
                monthly_housing_payment: LogNormalF64::{ location: 7.0, scale: 0.8 },
                years_at_address: UniformU8::{ low: 0, high: 20 }
            },
            
            // Underwriting decision
            underwriting: {
                decision: Uniform::{ choices: ["approved", "approved", "declined", "conditional"] },
                approved_amount: LogNormalF64::{ location: 8.8, scale: 1.4, optional: 0.3 },
                approved_rate: UniformF64::{ low: 0.03, high: 0.25, optional: 0.3 },
                approved_term: UniformU16::{ low: 12, high: 360, optional: 0.3 },
                
                // Decision factors
                risk_grade: Uniform::{ choices: ["A", "B", "C", "D", "E"] },
                decision_factors: UniformArray::{
                    min_size: 1,
                    max_size: 5,
                    element_type: Uniform::{ choices: [
                        "credit_score", "income", "debt_ratio", "employment_history", 
                        "collateral", "payment_history", "loan_purpose"
                    ]}
                },
                
                underwriter_id: UUID,
                decision_date: Instant,
                conditions: LoremIpsum::{ min_words: 5, max_words: 30, optional: 0.7 }
            }
        }
    },
    
    // Loan payments (for approved loans)
    loan_payments: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: days::30 },  // Monthly payments
        $data: {
            payment_id: UUID,
            loan_id: UUID,
            customer_id: Uniform::{ choices: $customer_ids },
            payment_date: Instant,
            
            // Payment details
            scheduled_amount: LogNormalF64::{ location: 6.0, scale: 0.8 },
            actual_amount: LogNormalF64::{ location: 6.0, scale: 0.8 },
            principal_amount: LogNormalF64::{ location: 5.8, scale: 0.8 },
            interest_amount: LogNormalF64::{ location: 5.5, scale: 0.7 },
            
            // Payment status
            status: Uniform::{ choices: ["on_time", "on_time", "late", "missed"] },
            days_late: UniformU8::{ low: 0, high: 90, optional: 0.8 },
            late_fee: UniformDecimal::{ low: 0.00, high: 50.00, optional: 0.8 },
            
            // Remaining loan balance
            remaining_balance: LogNormalF64::{ location: 8.5, scale: 1.2 },
            remaining_payments: UniformU16::{ low: 0, high: 300 },
            
            // Payment processing
            payment_method: Uniform::{ choices: ["autopay", "online", "check", "phone", "branch"] },
            processing_fee: UniformDecimal::{ low: 0.00, high: 10.00, optional: 0.6 }
        }
    }
}
```

**Generate credit and lending data:**
```bash
partiql-beamline-cli gen data \
    --seed 600 \
    --start-auto \
    --sample-count 60 \
    --script-path credit-lending.ion \
    --dataset credit_applications \
    --output-format ion-pretty
```

## Part 7: Portfolio and Investment Management

Create investment portfolio data with realistic asset allocations.

### Investment Portfolio System

Create `investment-portfolios.ion`:

```ion
rand_processes::{
    $n_customers: UniformU8::{ low: 25, high: 100 },
    $customer_ids: $n_customers::[UUID::()],
    
    // Customer investment profiles (static)
    investment_profiles: static_data::{
        $data: {
            customer_id: Uniform::{ choices: $customer_ids },
            profile_id: UUID,
            
            // Investment characteristics
            risk_tolerance: Uniform::{ choices: ["conservative", "moderate", "aggressive"] },
            investment_goal: Uniform::{ choices: [
                "retirement", "wealth_building", "income", "speculation", "preservation"
            ]},
            time_horizon_years: UniformU8::{ low: 1, high: 40 },
            
            // Portfolio allocation targets
            target_allocation: {
                stocks_percent: UniformF64::{ low: 0.2, high: 0.8 },
                bonds_percent: UniformF64::{ low: 0.1, high: 0.6 },
                cash_percent: UniformF64::{ low: 0.05, high: 0.3 },
                alternatives_percent: UniformF64::{ low: 0.0, high: 0.2 }
            },
            
            // Account details
            account_value: LogNormalF64::{ location: 10.0, scale: 1.5 },  // $10K-$10M range
            inception_date: Date,
            managed: Bool::{ p: 0.6 },  // 60% professionally managed
            advisor_id: UUID
        }
    },
    
    // Portfolio transactions (rebalancing, deposits, withdrawals)
    portfolio_transactions: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: days::UniformU8::{ low: 7, high: 60 } },
        $data: {
            transaction_id: UUID,
            portfolio_id: UUID,
            customer_id: Uniform::{ choices: $customer_ids },
            timestamp: Instant,
            
            // Transaction type
            transaction_type: Uniform::{ choices: [
                "deposit", "withdrawal", "rebalance", "dividend_reinvestment",
                "buy", "sell", "transfer", "fee_deduction"
            ]},
            
            // Security details
            symbol: Regex::{ pattern: "[A-Z]{3,4}", optional: 0.3 },  // Not all transactions have securities
            asset_class: Uniform::{ choices: ["equity", "bond", "cash", "alternative", "derivative"] },
            
            // Financial details
            shares: LogNormalF64::{ location: 3.0, scale: 1.5, optional: 0.3 },
            price_per_share: LogNormalF64::{ location: 3.2, scale: 0.8, optional: 0.3 },
            total_amount: LogNormalF64::{ location: 5.5, scale: 1.2 },
            
            // Portfolio impact
            portfolio_value_before: LogNormalF64::{ location: 10.0, scale: 1.4 },
            portfolio_value_after: LogNormalF64::{ location: 10.0, scale: 1.4 },
            
            // Performance tracking
            performance_impact: {
                return_contribution: NormalF64::{ mean: 0.0, std_dev: 0.02 },  // Daily returns
                risk_contribution: UniformF64::{ low: 0.0, high: 0.1 },
                sector_exposure_change: NormalF64::{ mean: 0.0, std_dev: 0.05 }
            }
        }
    }
}
```

## Part 8: Regulatory Compliance and Reporting

Financial institutions require extensive compliance and reporting.

### Compliance Monitoring System

Create `compliance-system.ion`:

```ion
rand_processes::{
    // Regulatory examinations
    examinations: rand_process::{
        $arrival: HomogeneousPoisson::{ interarrival: days::UniformU16::{ low: 30, high: 180 } },
        $data: {
            examination_id: UUID,
            examination_date: Instant,
            
            // Examination details
            regulator: Uniform::{ choices: ["FDIC", "OCC", "Federal_Reserve", "CFPB", "SEC"] },
            examination_type: Uniform::{ choices: [
                "safety_soundness", "consumer_compliance", "aml_bsa", 
                "fair_lending", "cra", "capital_adequacy"
            ]},
            
            // Scope and findings
            scope: LoremIpsum::{ min_words: 10, max_words: 50 },
            findings_count: UniformU8::{ low: 0, high: 15 },
            severity_level: Uniform::{ choices: ["satisfactory", "needs_improvement", "deficient"] },
            
            // Required actions
            corrective_actions: UniformArray::{
                min_size: 0,
                max_size: 8,
                element_type: LoremIpsum::{ min_words: 5, max_words: 20 }
            },
            due_date: Instant,
            completed: Bool::{ p: 0.8 },
            
            // Financial impact
            penalties: UniformDecimal::{ low: 0.00, high: 500000.00, optional: 0.7 },
            remediation_cost: LogNormalF64::{ location: 9.0, scale: 1.5, optional: 0.5 }
        }
    }
}
```

## Part 9: Complete Financial Database

Generate a comprehensive financial institution database.

### Full Banking System

Create `complete-banking.ion` combining all components:

```bash
# Generate complete financial database
partiql-beamline-cli gen db beamline-lite \
    --seed 1000 \
    --start-iso "2024-01-01T00:00:00Z" \
    --script-path fraud-detection.ion \
    --sample-count 25000 \
    --catalog-name financial-system-db \
    --catalog-path ./financial-database

# Examine the generated database
tree financial-database/financial-system-db/

# Check compliance report schemas
cat financial-database/financial-system-db/compliance_reports.shape.sql
cat financial-database/financial-system-db/suspicious_transactions.shape.sql
```

## Part 10: Financial Analytics Queries

Generate queries for financial analysis and regulatory reporting.

### Risk and Compliance Queries

```bash
# Generate regulatory compliance queries
partiql-beamline-cli query basic \
    --seed 1100 \
    --start-auto \
    --script-path fraud-detection.ion \
    --sample-count 8 \
    rand-select-all-fw \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 3 \
        --tbl-flt-path-depth-max 3 \
        --tbl-flt-pathstep-internal-all \
        --tbl-flt-pathstep-final-project \
        --tbl-flt-type-final-scalar \
        --pred-all
```

**Example generated queries:**
```sql
-- High-risk transactions for review
SELECT * FROM suspicious_transactions AS st 
WHERE (st.risk_score > 0.8 AND st.investigation.fraud_confirmed = true)

-- Large cash transactions requiring CTR filing
SELECT * FROM high_value_transactions AS hvt 
WHERE (hvt.amount >= 10000.0 AND hvt.compliance.ctr_filed = false)

-- Customer risk profiling
SELECT * FROM normal_transactions AS nt 
WHERE (nt.fraud_indicators.velocity_anomaly = true 
       OR nt.fraud_indicators.unusual_location = true)

-- Compliance report tracking
SELECT * FROM compliance_reports AS cr 
WHERE (cr.regulatory.priority = 'immediate' 
       AND cr.regulatory.requires_follow_up = true)
```

### Investment Analysis Queries

```bash
# Generate portfolio analysis queries
partiql-beamline-cli query basic \
    --seed 1200 \
    --start-auto \
    --script-path investment-portfolios.ion \
    --sample-count 6 \
    rand-sfw \
        --project-rand-min 2 \
        --project-rand-max 4 \
        --project-path-depth-max 2 \
        --project-pathstep-internal-all \
        --project-pathstep-final-all \
        --project-type-final-scalar \
        --tbl-flt-rand-min 1 \
        --tbl-flt-rand-max 2 \
        --pred-all
```

## Part 11: Schema Analysis

Examine the financial data schemas:

```bash
# Analyze financial transaction schema
partiql-beamline-cli infer-shape \
    --seed 500 \
    --start-auto \
    --script-path fraud-detection.ion \
    --output-format basic-ddl
```

**Expected DDL output:**
```sql
-- Dataset: normal_transactions
"account_id" VARCHAR,
"amount" DOUBLE,
"fraud_indicators" STRUCT<"merchant_risk": BOOL,"unusual_amount": BOOL,"unusual_location": BOOL,"unusual_time": BOOL,"velocity_anomaly": BOOL>,
"location" VARCHAR,
"merchant" VARCHAR,
"risk_score" DOUBLE,
"timestamp" TIMESTAMP,
"transaction_id" VARCHAR,
"transaction_outcome" VARCHAR

-- Dataset: suspicious_transactions  
"account_id" VARCHAR,
"amount" UNION<DOUBLE,DECIMAL(5, 2)>,
"fraud_indicators" STRUCT<"amount_anomaly": BOOL,"device_anomaly": BOOL,"foreign_country": BOOL,"high_risk_merchant": BOOL,"time_anomaly": BOOL,"unusual_location": BOOL,"velocity_anomaly": BOOL>,
"investigation" STRUCT<"account_blocked": BOOL,"customer_contacted": BOOL,"false_positive": BOOL,"flagged_by_system": BOOL,"fraud_confirmed": BOOL,"investigation_status": VARCHAR,"law_enforcement_notified": BOOL,"manual_review_required": BOOL>,
"risk_score" DOUBLE,
"timestamp" TIMESTAMP,
"transaction_id" VARCHAR,
"transaction_outcome" VARCHAR
```

## Key Financial Concepts Demonstrated

Through this tutorial, you've learned:

1. **Realistic Financial Distributions**: Using log-normal for amounts, normal for scores
2. **Risk Modeling**: Appropriate distributions for fraud and credit risk
3. **Regulatory Compliance**: Modeling reporting requirements and thresholds
4. **Customer Segmentation**: Different financial profiles and behaviors
5. **Transaction Patterns**: Normal vs. suspicious activity modeling
6. **Market Data**: High-frequency trading and market information
7. **Credit Analysis**: Loan applications, underwriting, and payment patterns
8. **Investment Management**: Portfolio transactions and performance tracking
9. **Fraud Detection**: Anomaly patterns and investigation workflows
10. **Compliance Reporting**: Regulatory filings and examination processes

## Best Practices for Financial Data

### 1. Use Appropriate Statistical Distributions

```ion
// Income, account balances, loan amounts - log-normal distribution
income: LogNormalF64::{ location: 10.5, scale: 0.8 },

// Credit scores, test scores - normal distribution  
credit_score: NormalF64::{ mean: 700.0, std_dev: 100.0 },

// Risk scores - Weibull distribution (most low risk, few high risk)
risk_score: WeibullF64::{ shape: 0.5, scale: 0.3 },
```

### 2. Model Realistic Compliance Patterns

```ion
// Most transactions are normal, few are suspicious
normal_rate: HomogeneousPoisson::{ interarrival: hours::6 },
suspicious_rate: HomogeneousPoisson::{ interarrival: days::30 },

// Compliance thresholds
ctr_threshold: 10000.00,  // $10K CTR reporting
high_risk_score: 0.8,    // Risk score triggering review
```

### 3. Use Realistic Nullability for Financial Data

```ion
// Required regulatory fields
customer_id: UUID::{ nullable: false, optional: false },

// Optional fields that might be missing
middle_name: LoremIpsumTitle::{ optional: 0.4 },

// Fields that might be null due to data quality issues
secondary_phone: Regex::{ pattern: "[0-9]{10}", nullable: 0.15, optional: 0.3 },
```

## Next Steps and Extensions

Expand your financial modeling:

1. **Cryptocurrency**: Digital asset trading, wallet transactions, DeFi protocols
2. **Insurance**: Claims processing, premium payments, underwriting
3. **Foreign Exchange**: Currency trading, cross-border payments, hedging
4. **Derivatives**: Options, futures, swaps trading and risk management
5. **Wealth Management**: High-net-worth customer patterns, private banking
6. **Commercial Banking**: Business accounts, commercial loans, cash management
7. **Regulatory Stress Testing**: Scenario generation for stress tests
8. **Anti-Money Laundering**: Complex layering and integration patterns

This tutorial demonstrates how PartiQL Beamline's statistical distributions and temporal modeling capabilities are particularly well-suited for financial data, where realistic patterns are crucial for effective testing, risk management, and regulatory compliance.

## Database Generation

Create a complete financial analytics database:

```bash
# Generate comprehensive financial database with all datasets
partiql-beamline-cli gen db beamline-lite \
    --seed 2000 \
    --start-iso "2024-01-01T00:00:00Z" \
    --script-path fraud-detection.ion \
    --sample-count 50000 \
    --catalog-name financial-analytics-db

echo "Financial database generated with:"
echo "- Account and customer data"
echo "- Normal and suspicious transactions" 
echo "- Compliance reports and investigations"
echo "- Risk scores and fraud indicators"
echo "- Complete schemas in Ion and SQL formats"
```

This comprehensive financial tutorial showcases PartiQL Beamline's ability to model complex, regulated industries where statistical accuracy and realistic patterns are essential for effective testing and analysis.
