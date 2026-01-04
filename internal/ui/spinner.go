package ui

import (
	"context"
	"fmt"
	"os"
	"time"

	"github.com/charmbracelet/bubbles/spinner"
	tea "github.com/charmbracelet/bubbletea"
	"golang.org/x/term"
)

// spinnerModel is a simple spinner model for running tasks.
type spinnerModel struct {
	spinner  spinner.Model
	message  string
	done     bool
	err      error
	quitting bool
}

type taskDoneMsg struct {
	err error
}

func newSpinnerModel(message string) spinnerModel {
	s := spinner.New()
	s.Spinner = spinner.Dot
	s.Style = SpinnerStyle // Uses SpinnerStyle from styles.go
	return spinnerModel{
		spinner: s,
		message: message,
	}
}

func (m spinnerModel) Init() tea.Cmd {
	return m.spinner.Tick
}

func (m spinnerModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		if msg.String() == "ctrl+c" {
			m.quitting = true
			return m, tea.Quit
		}
	case taskDoneMsg:
		m.done = true
		m.err = msg.err
		return m, tea.Quit
	case spinner.TickMsg:
		var cmd tea.Cmd
		m.spinner, cmd = m.spinner.Update(msg)
		return m, cmd
	}
	return m, nil
}

func (m spinnerModel) View() string {
	if m.quitting {
		return ""
	}
	if m.done {
		if m.err != nil {
			return RenderError(m.message) + "\n"
		}
		return RenderSuccess(m.message) + "\n"
	}
	return m.spinner.View() + " " + m.message
}

// RunWithSpinner runs a function with a spinner, showing progress.
// Returns the error from the function, or context.Canceled if user pressed ctrl+c.
func RunWithSpinner(ctx context.Context, message string, fn func() error) error {
	// If not a terminal, just run without spinner
	if !term.IsTerminal(int(os.Stdout.Fd())) {
		fmt.Printf("  %s...\n", message)
		return fn()
	}

	model := newSpinnerModel(message)

	// Create program with output to stderr to not interfere with stdout
	p := tea.NewProgram(model, tea.WithOutput(os.Stderr))

	// Run the task in a goroutine
	go func() {
		err := fn()
		p.Send(taskDoneMsg{err: err})
	}()

	// Run the spinner
	finalModel, err := p.Run()
	if err != nil {
		return fmt.Errorf("spinner error: %w", err)
	}

	final := finalModel.(spinnerModel)
	if final.quitting {
		return context.Canceled
	}

	return final.err
}

// RunStepsWithSpinner runs multiple steps with spinners sequentially.
type Step struct {
	Message string
	Run     func() error
}

func RunSteps(ctx context.Context, steps []Step) error {
	for _, step := range steps {
		if err := RunWithSpinner(ctx, step.Message, step.Run); err != nil {
			return err
		}
	}
	return nil
}

// SimpleSpinner provides a simpler API for showing progress without bubble tea.
// Use this when you need more control or are already in a TUI context.
type SimpleSpinner struct {
	frames   []string
	index    int
	message  string
	interval time.Duration
	stop     chan struct{}
	done     chan struct{}
}

// NewSimpleSpinner creates a new simple spinner.
func NewSimpleSpinner(message string) *SimpleSpinner {
	return &SimpleSpinner{
		frames:   []string{"⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"},
		message:  message,
		interval: 80 * time.Millisecond,
		stop:     make(chan struct{}),
		done:     make(chan struct{}),
	}
}

// Start begins the spinner animation.
func (s *SimpleSpinner) Start() {
	if !term.IsTerminal(int(os.Stdout.Fd())) {
		fmt.Printf("  %s...\n", s.message)
		return
	}

	go func() {
		defer close(s.done)
		for {
			select {
			case <-s.stop:
				return
			default:
				fmt.Fprintf(os.Stderr, "\r%s %s", SpinnerStyle.Render(s.frames[s.index]), s.message)
				s.index = (s.index + 1) % len(s.frames)
				time.Sleep(s.interval)
			}
		}
	}()
}

// Stop stops the spinner and shows success.
func (s *SimpleSpinner) Stop(success bool) {
	if !term.IsTerminal(int(os.Stdout.Fd())) {
		return
	}

	close(s.stop)
	<-s.done

	// Clear the line and show result
	fmt.Fprintf(os.Stderr, "\r\033[K") // Clear line
	if success {
		fmt.Fprintf(os.Stderr, "%s\n", RenderSuccess(s.message))
	} else {
		fmt.Fprintf(os.Stderr, "%s\n", RenderError(s.message))
	}
}
