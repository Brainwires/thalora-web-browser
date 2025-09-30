# Thalora MCP Form Testing Demo

This demo showcases Thalora's powerful web form interaction capabilities using the TwoLogs form test page as a real-world example.

## Overview

Thalora is a headless web browser designed for AI model integration via the Model Context Protocol (MCP). This demo demonstrates how AI agents can interact with web forms programmatically, which is essential for web automation, testing, and data collection tasks.

## Demo Scenario

We'll use the TwoLogs form test page (https://www.twologs.com/en/resources/formtest.asp) which provides a realistic testing environment for form manipulation and security testing.

### Key Features Demonstrated

1. **Form Discovery**: Automatically detect and analyze form structures
2. **Content Extraction**: Extract form data and metadata
3. **Field Manipulation**: Modify form fields programmatically
4. **Input Validation Testing**: Test various input scenarios
5. **Security Assessment**: Demonstrate potential form vulnerabilities

## MCP Commands for Form Testing

### 1. Basic Page Scraping and Form Discovery

First, let's scrape the form test page to understand its structure:

```bash
echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "scrape",
    "arguments": {
      "url": "https://www.twologs.com/en/resources/formtest.asp",
      "wait_for_js": true,
      "extract_links": true,
      "extract_metadata": true
    }
  }
}' | ./target/release/thalora
```

**What this shows:**
- Discovers the form with 6 input fields
- Extracts form structure including field types (text, checkbox, submit)
- Shows JavaScript processing capabilities
- Demonstrates form field analysis (scriptaddress, replacetext, etc.)

### 2. Targeted Form Content Extraction

Extract specific form elements using CSS selectors:

```bash
echo '{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "scrape_content_by_selector",
    "arguments": {
      "url": "https://www.twologs.com/en/resources/formtest.asp",
      "selectors": {
        "form_inputs": "input[type=text], input[type=checkbox], input[type=submit]",
        "form_labels": "label",
        "form_descriptions": "p, div",
        "required_fields": "input[required], *[aria-required]"
      }
    }
  }
}' | ./target/release/thalora
```

**What this demonstrates:**
- Precise field targeting using CSS selectors
- Identification of different input types
- Discovery of form validation requirements
- Extraction of form documentation and descriptions

### 3. Readable Content Analysis

Extract the human-readable content to understand form purpose and usage:

```bash
echo '{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "scrape_readable_content",
    "arguments": {
      "url": "https://www.twologs.com/en/resources/formtest.asp",
      "format": "markdown",
      "include_metadata": true
    }
  }
}' | ./target/release/thalora
```

**What this shows:**
- Clean extraction of form instructions and documentation
- Understanding of form security implications
- Context about spam prevention and form vulnerabilities
- Formatted output suitable for AI processing

### 4. Structured Data Extraction

Extract tables, lists, and structured content from the page:

```bash
echo '{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "scrape_structured_content",
    "arguments": {
      "url": "https://www.twologs.com/en/resources/formtest.asp",
      "content_types": ["lists", "metadata", "code_blocks"]
    }
  }
}' | ./target/release/thalora
```

**What this demonstrates:**
- Extraction of form option lists
- Metadata about form security testing
- Code snippets related to form manipulation

## Real-World Applications

This demo illustrates several practical AI use cases:

### 1. **Security Testing**
- Automated form vulnerability assessment
- Input validation testing
- Hidden field discovery
- Spam relay detection

### 2. **Web Automation**
- Form field population
- Multi-step form workflows
- Data extraction from dynamic forms
- Form submission testing

### 3. **Content Analysis**
- Understanding form purpose and requirements
- Extracting form documentation
- Identifying required vs optional fields
- Analyzing form accessibility features

### 4. **Research and Monitoring**
- Tracking form changes over time
- Comparing form implementations
- Gathering form interaction data
- Monitoring form availability

## Advanced Features Showcased

### JavaScript Processing
- Thalora can execute safe JavaScript to understand dynamic forms
- Handles client-side form validation
- Processes form modification scripts
- Manages browser-like behavior

### Human-like Interaction
- Adds realistic delays to mimic human behavior
- Processes forms as a real browser would
- Handles both static and dynamic content
- Supports session management for multi-page forms

### Security-Aware Processing
- Identifies potential security vulnerabilities
- Understands form manipulation techniques
- Recognizes spam relay risks
- Provides defensive security insights

## Technical Implementation

The demo leverages Thalora's core capabilities:

- **Boa JavaScript Engine**: For executing page scripts safely
- **HTTP/2 Client**: For efficient web requests
- **DOM Parser**: For form structure analysis
- **CSS Selector Engine**: For precise content targeting
- **Readability Algorithms**: For content extraction
- **MCP Protocol**: For AI model integration

## Running the Demo

1. Build Thalora: `cargo build --release`
2. Run any of the command examples above
3. Observe the rich form analysis output
4. Experiment with different selector combinations
5. Test various content extraction formats

This demo shows how Thalora enables AI models to interact with web forms intelligently, safely, and effectively - a crucial capability for modern AI applications that need to work with web content.