---
sidebar_position: 5
title: New Features Guide
---

# New Features Guide

This guide covers the latest enhancements to Phlow, including improved module syntax and advanced code block support.

## Enhanced Module Syntax

### Automatic Transformation

Phlow now automatically transforms legacy module syntax to the new standardized format during processing. This ensures backward compatibility while encouraging the use of the new, more consistent syntax.

#### How It Works

When you write:
```phlow
steps:
  - log:
      message: "Hello World"
      level: info
```

Phlow automatically transforms it to:
```phlow
steps:
  - use: log
    input:
      message: "Hello World"
      level: info
```

#### Transformation Rules

The transformation applies to:
- ✅ Properties in the root of `steps` arrays
- ✅ Properties in the root of `then` blocks  
- ✅ Properties in the root of `else` blocks
- ❌ Properties inside `payload` (ignored)
- ❌ Properties inside existing `input` blocks (ignored)
- ❌ Exclusive properties: `use`, `to`, `id`, `label`, `assert`, `condition`, `return`, `payload`, `input`, `then`, `else`

#### Examples

**Valid Transformations:**
```phlow
steps:
  - log:                    # ✅ Transformed
      message: "Step 1"
  - condition:
      assert: !phs true
    then:
      - cache:              # ✅ Transformed  
          action: set
      - use: postgres       # ❌ Already correct format
        input:
          query: "SELECT 1"
    else:
      - log:                # ✅ Transformed
          message: "Step 2"
```

**Ignored Cases:**
```phlow
steps:
  - payload:
      nested:
        log:                # ❌ Not transformed (inside payload)
          message: "ignored"
  - use: cache
    input:
      log:                  # ❌ Not transformed (inside input)
        message: "ignored"
```

## Advanced Code Blocks

### Multi-line Code with `!phs {}`

You can now write complex, multi-line code blocks using curly braces. These blocks are automatically processed and unified into single lines.

#### Basic Example

**Input:**
```phlow
- payload: !phs {
    let user = main.user;
    let age = user.age || 0;
    let category = age >= 18 ? "adult" : "minor";
    
    {
      name: user.name,
      age: age,
      category: category,
      timestamp: new Date().toISOString()
    }
  }
```

**Processed Output:**
```phlow
- payload: "{{ { let user = main.user; let age = user.age || 0; let category = age >= 18 ? \"adult\" : \"minor\"; { name: user.name, age: age, category: category, timestamp: new Date().toISOString() } } }}"
```

#### Complex Data Processing

```phlow
- payload: !phs {
    let orders = main.orders || [];
    
    let processed = orders.map(order => {
      let total = order.items.reduce((sum, item) => 
        sum + (item.price * item.quantity), 0
      );
      
      let discount = order.customer?.tier === "premium" ? 0.1 : 0;
      let finalTotal = total * (1 - discount);
      
      return {
        id: order.id,
        customerId: order.customer.id,
        itemCount: order.items.length,
        subtotal: total,
        discount: total * discount,
        total: finalTotal,
        status: finalTotal > 0 ? "valid" : "invalid"
      };
    });
    
    {
      orders: processed,
      summary: {
        total: processed.length,
        valid: processed.filter(o => o.status === "valid").length,
        totalRevenue: processed.reduce((sum, o) => sum + o.total, 0)
      }
    }
  }
```

#### With Module Calls

```phlow
- use: log
  input:
    level: "info"
    message: !phs {
      let stats = payload.summary;
      let revenue = Math.round(stats.totalRevenue * 100) / 100;
      
      `Processed ${stats.total} orders, ${stats.valid} valid, $${revenue} revenue`
    }
```

### Code Block Limitations

#### What You CAN Do:
- ✅ Variable declarations (`let`, `const`)
- ✅ Complex calculations and transformations
- ✅ Array and object manipulations
- ✅ Conditional expressions and ternary operators
- ✅ Method calls on existing objects
- ✅ Access to flow variables (`main`, `payload`, `steps`)

#### What You CANNOT Do:
- ❌ Function declarations (`function myFunc() {}`)
- ❌ Class declarations (`class MyClass {}`)
- ❌ Import statements (`import` or `require`)
- ❌ Async/await patterns

#### For Complex Functions, Use `!import`:

**complex-logic.phs:**
```javascript
function calculateTax(amount, state) {
  const taxRates = {
    "CA": 0.0875,
    "NY": 0.08,
    "TX": 0.0625
  };
  
  return amount * (taxRates[state] || 0.05);
}

function processOrder(order, state) {
  let subtotal = order.items.reduce((sum, item) => 
    sum + (item.price * item.quantity), 0
  );
  
  let tax = calculateTax(subtotal, state);
  
  return {
    subtotal: subtotal,
    tax: tax,
    total: subtotal + tax
  };
}

// Export the main function
processOrder
```

**main.phlow:**
```phlow
steps:
  - payload: !import complex-logic.phs
  - payload: !phs payload(main.order, main.state)
```

## Migration Guide

### Gradual Migration

You don't need to update all your flows at once. Both syntaxes work seamlessly together:

```phlow
modules:
  - module: log
  - module: cache
  
steps:
  # Keep existing legacy syntax
  - log:
      message: "Starting process"
      
  # Add new features gradually  
  - payload: !phs {
      let config = main.config;
      let processed = {
        ...config,
        timestamp: new Date().toISOString(),
        version: "2.0"
      };
      processed
    }
    
  # Mix with new syntax when convenient
  - use: cache
    input:
      action: set
      key: "config"
      value: !phs payload
      
  # Legacy syntax still works
  - log:
      level: "info"
      message: !phs `Configuration saved: ${payload.version}`
```

### Best Practices

1. **New Projects**: Use the new `use` + `input` syntax
2. **Existing Projects**: Migrate gradually, no rush needed
3. **Code Blocks**: Use for complex inline logic
4. **Functions**: Use `!import` for reusable functions
5. **Documentation**: Comment complex code blocks for maintainability

### Validation

All transformations are validated during processing. If there are issues, you'll see clear error messages:

```
❌ YAML Transformation Errors:
  1. Error including file ./handler.phlow: Missing required argument: 'target'

❌ Module transformation failed: Invalid module reference 'unknown_module'
```

## Performance Impact

- **Zero Runtime Cost**: Transformations happen during processing, not execution
- **Optimized Code**: Code blocks are optimized and cached
- **Backward Compatible**: No performance penalty for legacy syntax
- **Memory Efficient**: Transformed code uses the same memory footprint

The new features maintain Phlow's performance characteristics while providing enhanced developer experience and maintainability.
