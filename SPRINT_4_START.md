# 📋 Sprint 4: Testing & Quality - Planning

> **Status:** 📋 Planned  
> **Start Date:** 2026-02-12  
> **Duration:** 15 days (planned)  
> **Goal:** Increase test coverage to 50%+, add confidence for production deployment

---

## 📊 Context

### Current State
- **Sprint 1:** ✅ Complete (4/4 tasks) - Critical security fixes
- **Sprint 2:** ✅ Complete (4/4 tasks) - Simplification & refactoring
- **Sprint 3:** ✅ Complete (3/3 tasks) - Observability (OpenTelemetry, Distributed Tracing, Metrics)
- **Overall Progress:** 69% (11/16 tasks)
- **Architecture Score:** 9.3/10
- **Production Readiness:** 96%

### Test Coverage Analysis
- **Current Coverage:** ~36%
- **Target Coverage:** 50%+
- **Gap:** +14% needed
- **Critical Areas:** Integration flows, property-based tests, performance benchmarks

---

## 🎯 Sprint Objectives

### Primary Goals
1. **Integration Tests** - End-to-end testing for critical business flows
2. **Property-Based Tests** - Property verification for validators and state machines
3. **Performance Benchmarks** - Baseline metrics and regression detection
4. **Security Audit** - Final security review before production

### Success Metrics
- Test coverage: 36% → 50%+
- Integration tests: 0 → 10+ critical flows
- Property-based tests: 0 → 5+ properties
- Performance benchmarks: 0 → 5+ key operations

---

## 📝 Task List

### Task 4.1: Integration Tests 🔥 HIGH ROI

**Priority:** P1 Critical  
**Effort:** 5 days  
**ROI:** ⭐⭐⭐⭐⭐  
**Impact:** Critical for production confidence

#### Problem
- Test coverage only 36%
- No integration tests for critical flows
- Only unit-level components tested
- Risk of regression bugs in production

#### Solution
Write integration tests for end-to-end flows

#### Files to Create
```
apps/server/tests/integration/order_flow_test.rs (NEW)
apps/server/tests/integration/content_flow_test.rs (NEW)
apps/server/tests/integration/event_flow_test.rs (NEW)
crates/rustok-test-utils/src/fixtures.rs (NEW)
crates/rustok-test-utils/src/test_app.rs (NEW)
crates/rustok-test-utils/src/lib.rs (NEW)
```

#### Test Scenarios

##### Order Flow Test
```rust
#[tokio::test]
async fn test_complete_order_flow() {
    let app = spawn_test_app().await;
    
    // 1. Create product
    let product = app.create_product(ProductInput {
        title: "Test Product".into(),
        sku: "TEST-001".into(),
        price: 1000,
    }).await.unwrap();
    
    // 2. Create order
    let order = app.create_order(OrderInput {
        customer_id: test_customer_id(),
        items: vec![OrderItemInput {
            product_id: product.id,
            quantity: 2,
        }],
    }).await.unwrap();
    
    assert_eq!(order.status, OrderStatus::Draft);
    assert_eq!(order.total, 2000);
    
    // 3. Submit order
    let order = app.submit_order(order.id).await.unwrap();
    assert_eq!(order.status, OrderStatus::PendingPayment);
    
    // 4. Process payment
    let payment = app.process_payment(order.id, PaymentInput {
        method: PaymentMethod::Card,
        amount: 2000,
        card_token: "tok_test".into(),
    }).await.unwrap();
    
    assert!(payment.success);
    
    // 5. Verify order is paid
    let order = app.get_order(order.id).await.unwrap();
    assert_eq!(order.status, OrderStatus::Paid);
    
    // 6. Verify event was emitted
    let events = app.get_events_for_order(order.id).await;
    assert!(events.iter().any(|e| matches!(e, DomainEvent::OrderPaid { .. })));
    
    // 7. Verify read model updated
    let indexed_order = app.search_orders("TEST-001").await.unwrap();
    assert_eq!(indexed_order.len(), 1);
    assert_eq!(indexed_order[0].id, order.id);
}
```

##### Content Flow Test
```rust
#[tokio::test]
async fn test_content_lifecycle_flow() {
    let app = spawn_test_app().await;
    
    // 1. Create node
    let node = app.create_node(CreateNodeInput {
        kind: "article".into(),
        title: "Test Article".into(),
        body: Some(BodyInput {
            format: BodyFormat::Markdown,
            content: "# Test\n\nContent".into(),
        }),
    }).await.unwrap();
    
    assert_eq!(node.status, NodeStatus::Draft);
    
    // 2. Add translation
    let translation = app.add_translation(node.id, "ru", TranslationInput {
        title: "Тестовая статья".into(),
        body: "# Тест\n\nСодержание".into(),
    }).await.unwrap();
    
    // 3. Publish node
    let published = app.publish_node(node.id).await.unwrap();
    assert_eq!(published.status, NodeStatus::Published);
    assert!(published.published_at.is_some());
    
    // 4. Verify event emitted
    let events = app.get_events_for_node(node.id).await;
    assert!(events.iter().any(|e| matches!(e, DomainEvent::NodePublished { .. })));
    
    // 5. Search published node
    let results = app.search_nodes("article").await.unwrap();
    assert!(results.iter().any(|n| n.id == node.id));
}
```

##### Event Propagation Test
```rust
#[tokio::test]
async fn test_event_propagation_flow() {
    let app = spawn_test_app().await;
    
    // 1. Create event listener
    let events = Arc::new(Mutex::new(Vec::new()));
    app.subscribe_to_events(events.clone());
    
    // 2. Trigger event
    let node = app.create_node(CreateNodeInput {
        kind: "article".into(),
        title: "Test".into(),
        body: None,
    }).await.unwrap();
    
    // 3. Wait for event propagation
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // 4. Verify event received
    let captured = events.lock().await;
    assert!(captured.iter().any(|e| matches!(e, DomainEvent::NodeCreated { .. })));
    
    // 5. Verify event persisted in outbox
    let outbox_events = app.get_outbox_events().await;
    assert!(outbox_events.iter().any(|e| e.event_type == "NodeCreated"));
    
    // 6. Verify event relayed to subscribers
    let relayed_events = app.get_relayed_events().await;
    assert!(relayed_events > 0);
}
```

#### Deliverables
- ✅ Test utilities module (fixtures, test_app)
- ✅ Order flow integration tests (3 scenarios)
- ✅ Content flow integration tests (3 scenarios)
- ✅ Event propagation tests (2 scenarios)
- ✅ CI/CD integration
- ✅ Test coverage: 36% → 45%

#### Completion Criteria
- [ ] Test utilities module created
- [ ] Order flow tests (create, submit, pay, ship)
- [ ] Content flow tests (create, translate, publish)
- [ ] Event propagation tests (publish, relay, subscribe)
- [ ] All tests pass consistently
- [ ] Tests run in CI/CD pipeline
- [ ] Test coverage increased by at least 9%

---

### Task 4.2: Property-Based Tests

**Priority:** P2 Nice-to-Have  
**Effort:** 3 days  
**ROI:** ⭐⭐⭐⭐  
**Impact:** Edge case coverage, confidence in validators

#### Problem
- Only example-based tests
- Edge cases not covered
- Manual test case generation
- Risk of invalid state transitions

#### Solution
Add property-based tests with `proptest`

#### Dependencies to Add
```toml
[dev-dependencies]
proptest = "1.4"
proptest-derive = "0.4"
```

#### Files to Create
```
crates/rustok-core/tests/property_tenant_validation.rs (NEW)
crates/rustok-core/tests/property_event_validation.rs (NEW)
crates/rustok-commerce/tests/property_order_state_machine.rs (NEW)
crates/rustok-content/tests/property_node_state_machine.rs (NEW)
```

#### Property Test Examples

##### Tenant Identifier Validation
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_tenant_identifier_always_valid(s in "[a-z0-9-]{1,64}") {
        let validator = TenantIdentifierValidator::new();
        prop_assume!(!RESERVED_SLUGS.contains(&s.as_str()));
        
        let result = validator.validate_slug(&s);
        prop_assert!(result.is_ok());
    }
    
    #[test]
    fn test_invalid_chars_rejected(s in ".{1,64}") {
        prop_assume!(s.contains(|c: char| !c.is_alphanumeric() && c != '-'));
        
        let validator = TenantIdentifierValidator::new();
        let result = validator.validate_slug(&s);
        prop_assert!(result.is_err());
    }
    
    #[test]
    fn test_max_length_enforced(s in "[a-z0-9-]{65,100}") {
        let validator = TenantIdentifierValidator::new();
        let result = validator.validate_slug(&s);
        prop_assert!(matches!(result, Err(TenantValidationError::TooLong)));
    }
    
    #[test]
    fn test_reserved_slugs_rejected(s in RESERVED_SLUGS_PROP) {
        let validator = TenantIdentifierValidator::new();
        let result = validator.validate_slug(&s);
        prop_assert!(matches!(result, Err(TenantValidationError::ReservedSlug)));
    }
}
```

##### Event Validation Properties
```rust
proptest! {
    #[test]
    fn test_node_created_validation_properties(
        node_id in any::<Uuid>(),
        kind in "[a-z]{1,64}",
        author_id in any::<Option<Uuid>>()
    ) {
        let event = DomainEvent::NodeCreated {
            node_id,
            kind: kind.clone(),
            author_id,
        };
        
        let result = event.validate();
        
        // Kind cannot be empty
        if kind.is_empty() {
            prop_assert!(matches!(result, Err(_)));
        }
        // Kind cannot exceed 64 chars
        else if kind.len() > 64 {
            prop_assert!(matches!(result, Err(_)));
        } else {
            prop_assert!(result.is_ok());
        }
    }
    
    #[test]
    fn test_order_created_validation_properties(
        order_id in any::<Uuid>(),
        customer_id in any::<Uuid>(),
        total in any::<i64>()
    ) {
        let event = DomainEvent::OrderCreated {
            order_id,
            customer_id,
            total,
            items: vec![],
        };
        
        let result = event.validate();
        
        // Total cannot be negative
        if total < 0 {
            prop_assert!(matches!(result, Err(_)));
        } else {
            prop_assert!(result.is_ok());
        }
    }
}
```

##### State Machine Properties
```rust
proptest! {
    #[test]
    fn test_order_state_machine_transitions_valid(
        sequence in prop::collection::vec(
            prop::sample::select(vec![0u8, 1, 2, 3, 4]), // Draft, Submit, Pay, Ship, Deliver
            1..=10
        )
    ) {
        let mut order = Order::<Draft>::new(test_customer_id(), vec![]);
        
        for action in sequence {
            match action {
                0 => {
                    // Submit (only from Draft)
                    if let Order::<Draft> { .. } = order {
                        order = order.submit();
                    }
                }
                1 => {
                    // Pay (only from PendingPayment)
                    if let Order::<PendingPayment> { .. } = order {
                        order = order.pay(Uuid::new_v4());
                    }
                }
                2 => {
                    // Ship (only from Paid)
                    if let Order::<Paid> { .. } = order {
                        order = order.ship("TEST-123".into());
                    }
                }
                3 => {
                    // Deliver (only from Shipped)
                    if let Order::<Shipped> { .. } = order {
                        order = order.deliver();
                    }
                }
                _ => {}
            }
        }
        
        // Final state is always valid
        // (Compile-time guarantees ensure this)
    }
}
```

#### Deliverables
- ✅ Proptest dependency added
- ✅ Tenant identifier property tests (4+ properties)
- ✅ Event validation property tests (3+ properties)
- ✅ Order state machine property tests (2+ properties)
- ✅ Node state machine property tests (2+ properties)
- ✅ Documentation (6KB)

#### Completion Criteria
- [ ] Proptest dependency configured
- [ ] Tenant identifier tests (valid, invalid, reserved, length)
- [ ] Event validation tests (all event types)
- [ ] State machine tests (valid transitions)
- [ ] All property tests pass
- [ ] Property tests run in CI/CD
- [ ] Documentation with examples

---

### Task 4.3: Performance Benchmarks

**Priority:** P2 Nice-to-Have  
**Effort:** 2 days  
**ROI:** ⭐⭐⭐  
**Impact:** Performance regression detection, optimization guidance

#### Problem
- No baseline for performance
- Refactorings may slow down the system
- No automatic performance tests
- Risk of performance regressions

#### Solution
Add benchmarks with `criterion`

#### Dependencies to Add
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }
```

#### Files to Create
```
benches/tenant_cache_bench.rs (NEW)
benches/event_bus_bench.rs (NEW)
benches/event_validation_bench.rs (NEW)
benches/state_machine_bench.rs (NEW)
```

#### Benchmark Examples

##### Tenant Cache Benchmark
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rustok_tenant::cache::TenantCache;
use rustok_tenant::Tenant;
use std::time::Duration;

fn bench_cache_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("tenant_cache_hit");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let cache = setup_cache_with_size(size);
            
            b.iter(|| {
                cache.get(black_box("test-tenant-1"))
            });
        });
    }
    
    group.finish();
}

fn bench_cache_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("tenant_cache_miss");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let cache = setup_cache_with_size(size);
            
            b.iter(|| {
                cache.get(black_box("non-existent-tenant"))
            });
        });
    }
    
    group.finish();
}

fn bench_cache_eviction(c: &mut Criterion) {
    let cache = TenantCache::new(1000, Duration::from_secs(3600));
    
    c.bench_function("cache_eviction_lru", |b| {
        b.iter(|| {
            // Fill cache beyond capacity
            for i in 0..1500 {
                cache.insert(&format!("tenant-{}", i), test_tenant());
            }
        });
    });
}

criterion_group!(
    benches,
    bench_cache_hit,
    bench_cache_miss,
    bench_cache_eviction
);
criterion_main!(benches);
```

##### EventBus Benchmark
```rust
fn bench_event_publish(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_bus_publish");
    
    for subscribers in [1, 10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(subscribers), subscribers, |b, &subscribers| {
            let (bus, _) = setup_bus_with_subscribers(subscribers);
            let event = test_event();
            
            b.iter(|| {
                bus.publish(black_box(event.clone()))
            });
        });
    }
    
    group.finish();
}

fn bench_event_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_bus_dispatch");
    
    for queue_size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(queue_size), queue_size, |b, &queue_size| {
            let (bus, mut dispatcher) = setup_bus_with_queue(queue_size);
            
            // Fill queue
            for _ in 0..queue_size {
                bus.publish(test_event()).await;
            }
            
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| dispatcher.dispatch());
        });
    }
    
    group.finish();
}

fn bench_event_validation(c: &mut Criterion) {
    c.bench_function("event_validate_node_created", |b| {
        let event = test_node_created_event();
        
        b.iter(|| {
            black_box(&event).validate()
        });
    });
    
    c.bench_function("event_validate_order_created", |b| {
        let event = test_order_created_event();
        
        b.iter(|| {
            black_box(&event).validate()
        });
    });
}
```

##### State Machine Benchmark
```rust
fn bench_state_machine_transitions(c: &mut Criterion) {
    c.bench_function("order_submit", |b| {
        let order = Order::<Draft>::new(test_customer_id(), vec![]);
        
        b.iter(|| {
            black_box(&order).submit()
        });
    });
    
    c.bench_function("order_pay", |b| {
        let order = Order::<Draft>::new(test_customer_id(), vec![]).submit();
        
        b.iter(|| {
            black_box(&order).pay(Uuid::new_v4())
        });
    });
    
    c.bench_function("node_publish", |b| {
        let node = Node::<Draft>::new("article", "Test", None);
        
        b.iter(|| {
            black_box(&node).publish()
        });
    });
}

fn bench_zero_overhead(c: &mut Criterion) {
    c.bench_function("state_machine_overhead", |b| {
        let order = Order::<Draft>::new(test_customer_id(), vec![]);
        let order: Box<dyn Any> = Box::new(order);
        
        b.iter(|| {
            // Typestate should have zero runtime overhead
            black_box(&order)
        });
    });
}
```

#### Deliverables
- ✅ Criterion dependency configured
- ✅ Tenant cache benchmarks (hit, miss, eviction)
- ✅ EventBus benchmarks (publish, dispatch, validation)
- ✅ State machine benchmarks (transitions, overhead)
- ✅ Baseline metrics established
- ✅ CI/CD integration (criterion reports)
- ✅ Documentation (8KB)

#### Completion Criteria
- [ ] Criterion dependency added
- [ ] Tenant cache benchmarks implemented
- [ ] EventBus benchmarks implemented
- [ ] State machine benchmarks implemented
- [ ] Baseline metrics recorded
- [ ] Benchmarks run in CI/CD
- [ ] Performance report generated

---

### Task 4.4: Security Audit

**Priority:** P1 Critical  
**Effort:** 3 days  
**ROI:** ⭐⭐⭐⭐⭐  
**Impact:** Production readiness, security confidence

#### Problem
- No formal security audit
- Potential vulnerabilities in critical paths
- Need for security review before production

#### Solution
Comprehensive security audit of codebase

#### Audit Checklist

##### Authentication & Authorization
- [ ] JWT token validation
- [ ] Token expiration handling
- [ ] RBAC permission checks
- [ ] Multi-tenant isolation
- [ ] Admin operation protections

##### Input Validation
- [ ] All API inputs validated
- [ ] SQL injection prevention
- [ ] XSS prevention
- [ ] Path traversal prevention
- [ ] Command injection prevention

##### Data Protection
- [ ] Sensitive data encryption
- [ ] Secrets management
- [ ] Database encryption at rest
- [ ] TLS/SSL configuration
- [ ] Secure headers (CORS, CSP, HSTS)

##### Event System
- [ ] Event validation enforcement
- [ ] Outbox pattern security
- [ ] Event relay authentication
- [ ] Replay attack prevention
- [ ] Event tampering detection

##### Infrastructure
- [ ] Dependency vulnerability scan (cargo audit)
- [ ] Docker image security
- [ ] Network isolation
- [ ] Rate limiting effectiveness
- [ ] Logging & monitoring security

##### Tenant Security
- [ ] Identifier validation (Sprint 1 Task 1.2)
- [ ] Tenant data isolation
- [ ] Cross-tenant access prevention
- [ ] Tenant cache security
- [ ] Multi-tenant query isolation

#### Deliverables
- ✅ Security audit report (15KB)
- ✅ Vulnerability findings (if any)
- ✅ Remediation recommendations
- ✅ Security score assessment
- ✅ Production security checklist

#### Completion Criteria
- [ ] All audit checklist items reviewed
- [ ] Vulnerability findings documented
- [ ] Critical vulnerabilities fixed
- [ ] Security report completed
- [ ] Production security checklist created

---

## 📊 Sprint Metrics

### Expected Deliverables
| Task | LOC | Tests | Docs | Effort |
|------|-----|-------|------|--------|
| 4.1: Integration Tests | 600+ | 10+ | 8KB | 5d |
| 4.2: Property Tests | 400+ | 15+ | 6KB | 3d |
| 4.3: Benchmarks | 300+ | 0 | 8KB | 2d |
| 4.4: Security Audit | 0 | 0 | 15KB | 3d |
| **Total** | **1300+** | **25+** | **37KB** | **13d** |

### Architecture Impact
```
Architecture Score: 9.3/10 → 9.5/10 (+0.2)
Production Readiness: 96% → 100% (+4%)
Test Coverage: 36% → 50%+ (+14%)
Security Score: 90% → 95% (+5%)
```

---

## 🚀 Success Criteria

### Must Have (P1)
- ✅ Integration tests for critical flows (order, content, events)
- ✅ Security audit completed
- ✅ Test coverage increased to 50%+
- ✅ All tests pass consistently
- ✅ CI/CD pipeline running all tests

### Should Have (P2)
- ✅ Property-based tests for validators
- ✅ Property-based tests for state machines
- ✅ Performance benchmarks established
- ✅ Baseline metrics recorded

### Nice to Have (P3)
- ✅ Fuzzing tests
- ✅ Chaos engineering tests
- ✅ Load testing framework
- ✅ Automated performance regression detection

---

## 📚 Documentation

### New Documentation Files
1. **SPRINT_4_START.md** (this file) - Sprint planning
2. **SPRINT_4_PROGRESS.md** (to be created) - Progress tracking
3. **SPRINT_4_COMPLETION.md** (to be created) - Completion report
4. **docs/INTEGRATION_TESTING_GUIDE.md** (to be created) - Testing guide
5. **docs/PROPERTY_TESTING_GUIDE.md** (to be created) - Proptest guide
6. **docs/PERFORMANCE_BENCHMARKS_GUIDE.md** (to be created) - Criterion guide
7. **docs/SECURITY_AUDIT_REPORT.md** (to be created) - Security findings

---

## 🔗 References

### Internal Documentation
- [ARCHITECTURE_IMPROVEMENT_PLAN.md](./ARCHITECTURE_IMPROVEMENT_PLAN.md) - Master plan
- [SPRINT_3_COMPLETION.md](./SPRINT_3_COMPLETION.md) - Previous sprint
- [SPRINT_2_COMPLETED.md](./SPRINT_2_COMPLETED.md) - Sprint 2 results
- [ARCHITECTURE_STATUS.md](./ARCHITECTURE_STATUS.md) - Current status

### External Resources
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Proptest Documentation](https://altsysrq.github.io/proptest-book/proptest/intro.html)
- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/index.html)
- [OWASP Security Checklist](https://owasp.org/www-project-web-security-testing-guide/)

---

## 📅 Timeline

**Week 1 (Days 1-5):**
- Day 1-2: Test utilities setup + Order flow tests
- Day 3-4: Content flow tests + Event propagation tests
- Day 5: CI/CD integration + Test coverage report

**Week 2 (Days 6-10):**
- Day 6-8: Property-based tests (validators + state machines)
- Day 9-10: Performance benchmarks

**Week 3 (Days 11-13):**
- Day 11-13: Security audit + documentation

**Sprint Review:** Day 13-15

---

**Sprint 4 Status:** 📋 Planned  
**Next Task:** Task 4.1 - Integration Tests  
**Start Date:** 2026-02-12
