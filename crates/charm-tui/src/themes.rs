//! Charm.sh-inspired themes and styling using owo-colors
//! 
//! Provides beautiful, consistent color schemes and styling for the poker game

use owo_colors::OwoColorize;
use ratatui::{
    style::{Color, Modifier, Style as RatatuiStyle},
    text::{Line, Span, Text},
};

/// Charm.sh-inspired color palette
#[derive(Debug, Clone)]
pub struct CharmPalette {
    // Primary colors (inspired by Charm.sh branding)
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    
    // Semantic colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // Poker-specific colors
    pub spades: Color,
    pub hearts: Color,
    pub diamonds: Color,
    pub clubs: Color,
    
    // UI colors
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
}

impl Default for CharmPalette {
    fn default() -> Self {
        Self {
            // Traditional casino colors - dark red and forest green
            primary: Color::Rgb(139, 69, 19),      // Dark red/brown (poker table felt)
            secondary: Color::Rgb(34, 139, 34),    // Forest green
            accent: Color::Rgb(255, 215, 0),       // Gold accents
            
            // Semantic colors
            success: Color::Rgb(34, 139, 34),      // Forest green
            warning: Color::Rgb(255, 140, 0),      // Dark orange
            error: Color::Rgb(178, 34, 34),        // Fire brick red
            info: Color::Rgb(70, 130, 180),        // Steel blue
            
            // Poker suit colors (traditional)
            spades: Color::Black,
            hearts: Color::Rgb(178, 34, 34),       // Dark red
            diamonds: Color::Rgb(178, 34, 34),     // Dark red  
            clubs: Color::Black,
            
            // Casino-style dark theme
            background: Color::Rgb(0, 50, 0),      // Very dark green (baize)
            surface: Color::Rgb(139, 69, 19),      // Dark red/brown
            border: Color::Rgb(255, 215, 0),       // Gold borders
            text_primary: Color::Rgb(255, 255, 255),   // Pure white
            text_secondary: Color::Rgb(220, 220, 220), // Light gray
            text_muted: Color::Rgb(169, 169, 169),     // Dark gray
        }
    }
}

/// Casino-style component styles for traditional poker
#[derive(Debug, Clone)]
pub struct CharmStyles {
    pub palette: CharmPalette,
}

impl CharmStyles {
    pub fn new() -> Self {
        Self {
            palette: CharmPalette::default(),
        }
    }
    
    /// Primary button style (casino gold on dark red)
    pub fn button_primary(&self) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(self.palette.accent)          // Gold text
            .bg(self.palette.surface)         // Dark red/brown background
            .add_modifier(Modifier::BOLD)
    }
    
    /// Secondary button style (forest green)
    pub fn button_secondary(&self) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(self.palette.text_primary)
            .bg(self.palette.secondary)       // Forest green background
            .add_modifier(Modifier::BOLD)
    }
    
    /// Card style for poker cards
    pub fn card(&self, suit: CardSuit) -> RatatuiStyle {
        let suit_color = match suit {
            CardSuit::Spades => self.palette.spades,
            CardSuit::Hearts => self.palette.hearts,
            CardSuit::Diamonds => self.palette.diamonds,
            CardSuit::Clubs => self.palette.clubs,
        };
        
        RatatuiStyle::default()
            .fg(suit_color)
            .bg(Color::White)           // White card background
            .add_modifier(Modifier::BOLD)
    }
    
    /// Pot/chip display style (gold for casino chips)
    pub fn chips(&self) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(self.palette.accent)        // Gold color for chips
            .add_modifier(Modifier::BOLD)
    }
    
    /// Player name style (gold for active, white for inactive)
    pub fn player_name(&self, is_active: bool) -> RatatuiStyle {
        if is_active {
            RatatuiStyle::default()
                .fg(self.palette.accent)     // Gold for active player
                .add_modifier(Modifier::BOLD)
        } else {
            RatatuiStyle::default()
                .fg(self.palette.text_secondary)
        }
    }
    
    /// Border style for boxes/panels (gold casino borders)
    pub fn border(&self) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(self.palette.border)        // Gold borders
    }
    
    /// Success message style
    pub fn success(&self) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(self.palette.success)
            .add_modifier(Modifier::BOLD)
    }
    
    /// Error message style  
    pub fn error(&self) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(self.palette.error)
            .add_modifier(Modifier::BOLD)
    }
    
    /// Warning message style
    pub fn warning(&self) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(self.palette.warning)
            .add_modifier(Modifier::BOLD)
    }
    
    /// Title/header style (gold casino titles)
    pub fn title(&self) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(self.palette.accent)        // Gold titles
            .add_modifier(Modifier::BOLD)
    }
    
    /// Subtitle style (forest green subtitles)
    pub fn subtitle(&self) -> RatatuiStyle {
        RatatuiStyle::default()
            .fg(self.palette.secondary)     // Forest green
            .add_modifier(Modifier::ITALIC)
    }
}

impl Default for CharmStyles {
    fn default() -> Self {
        Self::new()
    }
}

/// Card suit enum for styling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardSuit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

/// Helper function to create stylized text spans
pub fn stylized_text(text: &str, style: RatatuiStyle) -> Span {
    Span::styled(text, style)
}

/// Helper function to create Charm.sh-style gradient text
pub fn gradient_text(text: &str, start_color: Color, end_color: Color) -> Text {
    // Simple two-color gradient approximation
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    
    let mut spans = Vec::new();
    for (i, ch) in chars.iter().enumerate() {
        let ratio = if len > 1 { i as f32 / (len - 1) as f32 } else { 0.0 };
        
        // Simple color interpolation (this could be more sophisticated)
        let color = if ratio < 0.5 { start_color } else { end_color };
        
        spans.push(Span::styled(ch.to_string(), RatatuiStyle::default().fg(color)));
    }
    
    Text::from(Line::from(spans))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_charm_palette_creation() {
        let palette = CharmPalette::default();
        assert_eq!(palette.primary, Color::Rgb(255, 121, 198));
    }
    
    #[test]
    fn test_charm_styles_creation() {
        let styles = CharmStyles::new();
        let button_style = styles.button_primary();
        assert_eq!(button_style.bg, Some(styles.palette.primary));
    }
    
    #[test]
    fn test_card_suit_styling() {
        let styles = CharmStyles::new();
        let heart_style = styles.card(CardSuit::Hearts);
        assert_eq!(heart_style.fg, Some(styles.palette.hearts));
    }
}