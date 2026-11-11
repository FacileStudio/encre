package encre

import (
	"context"
	"os"
	"path/filepath"
)

// PDFOptions lays out a PDF. ImageDir, when set, is the only directory images are read from.
type PDFOptions struct {
	Title    string
	Footer   string
	ImageDir string
}

func (o PDFOptions) args(format Format, output string) []string {
	args := []string{"pdf", "--from", string(format), "--output=" + output}
	if o.Title != "" {
		args = append(args, "--title="+o.Title)
	}
	if o.Footer != "" {
		args = append(args, "--footer="+o.Footer)
	}
	if o.ImageDir != "" {
		args = append(args, "--image-dir="+o.ImageDir)
	}
	return args
}

// PDF renders a document in format to PDF bytes, through a temporary file.
func PDF(ctx context.Context, input []byte, format Format, options PDFOptions) ([]byte, error) {
	dir, err := os.MkdirTemp("", "encre-pdf-")
	if err != nil {
		return nil, err
	}
	defer os.RemoveAll(dir)
	output := filepath.Join(dir, "out.pdf")
	if _, err := Run(ctx, input, options.args(format, output)...); err != nil {
		return nil, err
	}
	return os.ReadFile(output)
}
