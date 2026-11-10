package encre

import (
	"bytes"
	"context"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func binary(t *testing.T) {
	t.Helper()
	if bin := os.Getenv("ENCRE_BIN"); bin != "" {
		if _, err := os.Stat(bin); err == nil {
			return
		}
	}
	for _, profile := range []string{"release", "debug"} {
		path, _ := filepath.Abs(filepath.Join("..", "target", profile, "encre"))
		if _, err := os.Stat(path); err == nil {
			t.Setenv("ENCRE_BIN", path)
			return
		}
	}
	t.Skip("no encre binary: run cargo build -p encre-cli or set ENCRE_BIN")
}

func TestParseSerializeRoundTrip(t *testing.T) {
	binary(t)
	ctx := context.Background()
	doc, err := Parse(ctx, "# Plan\n\n[badge:success Shipped] ==now==", true)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Contains(doc, []byte(`"type":"badge"`)) || !bytes.Contains(doc, []byte(`"id":`)) {
		t.Fatalf("parse: %s", doc)
	}
	markdown, err := Serialize(ctx, doc)
	if err != nil || markdown != "# Plan\n\n[badge:success Shipped] ==now==" {
		t.Fatalf("serialize: %q %v", markdown, err)
	}
}

func TestValidateReportsTheSourceLine(t *testing.T) {
	binary(t)
	verdict, err := Validate(context.Background(), []byte("ok\n\nbell \x07 here"), Markdown)
	if err != nil {
		t.Fatal(err)
	}
	if verdict.Valid || len(verdict.Errors) != 1 || verdict.Errors[0].Line != 3 || verdict.Errors[0].Column != 1 {
		t.Fatalf("verdict: %+v", verdict)
	}
	good, err := Validate(context.Background(), []byte("fine"), Markdown)
	if err != nil || !good.Valid {
		t.Fatalf("valid verdict: %+v %v", good, err)
	}
}

func TestRefusalsAreTypedErrors(t *testing.T) {
	binary(t)
	_, err := Serialize(context.Background(), []byte(`{"type":"doc","content":[]}`))
	var refused *Error
	if !errors.As(err, &refused) || !refused.Invalid() || refused.Code != "invalid" || refused.Path == "" {
		t.Fatalf("error: %#v", err)
	}
	_, err = Run(context.Background(), nil, "no-such-command")
	if !errors.As(err, &refused) || refused.Exit != 2 || refused.Invalid() {
		t.Fatalf("usage error: %#v", err)
	}
}

func TestExportsAndPDF(t *testing.T) {
	binary(t)
	ctx := context.Background()
	input := []byte("Release [status:done] and ==this==.\n\n```mermaid\nA --> B\n```")
	common, err := CommonMark(ctx, input, Markdown)
	if err != nil || !strings.Contains(common, `\[done\]`) || !strings.Contains(common, "```mermaid") {
		t.Fatalf("commonmark: %q %v", common, err)
	}
	plain, err := PlainText(ctx, input, Markdown)
	if err != nil || !strings.Contains(plain, "Release done and this.") {
		t.Fatalf("plain text: %q %v", plain, err)
	}
	pdf, err := PDF(ctx, input, Markdown, PDFOptions{Title: "Plan", Footer: "Facile"})
	if err != nil || !bytes.HasPrefix(pdf, []byte("%PDF-")) {
		t.Fatalf("pdf: %d bytes, %v", len(pdf), err)
	}
}

func TestMissingBinary(t *testing.T) {
	t.Setenv("ENCRE_BIN", filepath.Join(t.TempDir(), "absent"))
	if _, err := Run(context.Background(), nil, "parse"); err == nil {
		t.Fatal("a missing binary must fail")
	}
	if _, err := Run(context.Background(), nil); err == nil {
		t.Fatal("no command must fail")
	}
}
