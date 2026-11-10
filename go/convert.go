package encre

import (
	"context"
	"encoding/json"
	"fmt"
)

// Format names what an input holds, for the commands that read either.
type Format string

// The two input formats.
const (
	Markdown Format = "markdown"
	JSON     Format = "json"
)

// Problem is one reason a document is invalid. Line and Column are 1-based and set only for markdown input.
type Problem struct {
	Path    string `json:"path"`
	Message string `json:"message"`
	Line    int    `json:"line,omitempty"`
	Column  int    `json:"column,omitempty"`
}

// Verdict is the answer of Validate.
type Verdict struct {
	Valid  bool      `json:"valid"`
	Errors []Problem `json:"errors"`
}

func field(out []byte, key string) (string, error) {
	var body map[string]string
	if err := json.Unmarshal(out, &body); err != nil {
		return "", fmt.Errorf("encre: unexpected output: %w", err)
	}
	return body[key], nil
}

func text(ctx context.Context, input []byte, key string, args ...string) (string, error) {
	out, err := Run(ctx, input, args...)
	if err != nil {
		return "", err
	}
	return field(out, key)
}

// Validate checks a document in format. An invalid document is a Verdict, not an error.
func Validate(ctx context.Context, input []byte, format Format) (Verdict, error) {
	args := []string{"validate", "--from", string(format)}
	res, err := exec1(ctx, input, args)
	if err != nil {
		return Verdict{}, err
	}
	var verdict Verdict
	if (res.exit == 0 || res.exit == 1) && json.Unmarshal(res.stdout, &verdict) == nil {
		return verdict, nil
	}
	return Verdict{}, refusal("validate", res.stderr, res.exit)
}

// Parse reads Facile Markdown into encre JSON; ids gives every block a fresh id.
func Parse(ctx context.Context, markdown string, ids bool) (json.RawMessage, error) {
	args := []string{"parse"}
	if ids {
		args = append(args, "--ids")
	}
	out, err := Run(ctx, []byte(markdown), args...)
	return json.RawMessage(out), err
}

// Serialize writes an encre JSON document as canonical Facile Markdown.
func Serialize(ctx context.Context, doc []byte) (string, error) {
	return text(ctx, doc, "markdown", "serialize")
}

// CommonMark writes a document in format as portable CommonMark with GFM, lossy.
func CommonMark(ctx context.Context, input []byte, format Format) (string, error) {
	return text(ctx, input, "markdown", "commonmark", "--from", string(format))
}

// PlainText gives the text of a document in format, for search and previews.
func PlainText(ctx context.Context, input []byte, format Format) (string, error) {
	return text(ctx, input, "text", "plain-text", "--from", string(format))
}
