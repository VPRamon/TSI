# Frontend Refactoring Checklist

## ✅ Completed Infrastructure

- [x] Feature-based directory structure
- [x] Shared component library (TsiButton, TsiCard, TsiInput, TsiSpinner, TsiAlert)
- [x] Centralized API service with TypeScript types
- [x] Reusable composables (useAsync, usePolling)
- [x] Design token system (CSS variables)
- [x] Global styles architecture (tokens, base, utilities)
- [x] Path aliases configuration (@/, @shared/, @features/)
- [x] ESLint and Prettier configuration
- [x] Updated package.json with lint/format scripts
- [x] TypeScript configuration with strict mode
- [x] Comprehensive documentation (ARCHITECTURE.md, MIGRATION_GUIDE.md, etc.)

## ✅ Completed Features

- [x] **Upload Feature** (Example Implementation)
  - [x] UploadPage.vue with modern composition API
  - [x] FileUploadWidget component
  - [x] useFileUpload composable
  - [x] Feature-specific styles
  - [x] Full TypeScript integration

## 🔄 Remaining Feature Migrations

Follow the pattern established in `features/upload/`:

### Sky Map Feature
- [ ] Create `features/sky-map/` folder structure
- [ ] Migrate `SkyMap.vue` → `SkyMapPage.vue`
- [ ] Extract sky map components
- [ ] Create `useSkyMapData` composable
- [ ] Add feature-specific types
- [ ] Apply design tokens to styles
- [ ] Update router import path

### Distributions Feature
- [ ] Create `features/distributions/` folder structure
- [ ] Migrate `Distributions.vue` → `DistributionsPage.vue`
- [ ] Extract distribution components
- [ ] Create `useDistributions` composable
- [ ] Add feature-specific types
- [ ] Apply design tokens to styles
- [ ] Update router import path

### Insights Feature
- [ ] Create `features/insights/` folder structure
- [ ] Migrate `Insights.vue` → `InsightsPage.vue`
- [ ] Extract insights components (metrics cards, correlation heatmap)
- [ ] Create `useInsights` composable
- [ ] Add feature-specific types
- [ ] Apply design tokens to styles
- [ ] Update router import path

### Timeline Feature
- [ ] Create `features/timeline/` folder structure
- [ ] Migrate `ScheduledTimeline.vue` → `TimelinePage.vue`
- [ ] Extract timeline components
- [ ] Create `useTimeline` composable
- [ ] Add feature-specific types
- [ ] Apply design tokens to styles
- [ ] Update router import path

### Trends Feature
- [ ] Create `features/trends/` folder structure
- [ ] Migrate `Trends.vue` → `TrendsPage.vue`
- [ ] Extract trend components
- [ ] Create `useTrends` composable
- [ ] Add feature-specific types
- [ ] Apply design tokens to styles
- [ ] Update router import path

### Compare Feature
- [ ] Create `features/compare/` folder structure
- [ ] Migrate `CompareSchedules.vue` → `ComparePage.vue`
- [ ] Extract comparison components
- [ ] Create `useCompare` composable
- [ ] Add feature-specific types
- [ ] Apply design tokens to styles
- [ ] Update router import path

### Visibility Feature
- [ ] Create `features/visibility/` folder structure
- [ ] Migrate `VisibilityMap.vue` → `VisibilityPage.vue`
- [ ] Extract visibility map components
- [ ] Create `useVisibility` composable
- [ ] Add feature-specific types
- [ ] Apply design tokens to styles
- [ ] Update router import path

## 🧩 Shared Components to Migrate

### Navigation Component
- [ ] Move `Navigation.vue` to `shared/components/`
- [ ] Rename to `TsiNavigation.vue`
- [ ] Use composition API
- [ ] Add TypeScript types
- [ ] Use design tokens
- [ ] Update imports in App.vue

### Legacy Components Cleanup
- [ ] Audit `components/` folder
- [ ] Move reusable components to `shared/components/`
- [ ] Move feature-specific components to respective features
- [ ] Delete duplicated components
- [ ] Update all imports

## 🎨 Styling Enhancements

- [ ] Migrate remaining inline styles to design tokens
- [ ] Remove old `styles.css` file
- [ ] Consolidate feature-specific styles
- [ ] Add dark mode support (optional)
- [ ] Ensure responsive design across all features
- [ ] Test accessibility (ARIA labels, keyboard navigation)

## 📝 Router Updates

- [ ] Update all route imports to use lazy loading
- [ ] Update route paths to match new feature structure
- [ ] Add route meta (titles, auth requirements, etc.)
- [ ] Test all navigation flows

## 🧪 Testing (Future Work)

### Unit Tests
- [ ] Set up Vitest configuration
- [ ] Write tests for composables
  - [ ] useAsync
  - [ ] usePolling
  - [ ] useFileUpload
  - [ ] Feature-specific composables
- [ ] Write tests for utility functions
  - [ ] formatters
  - [ ] validators

### Component Tests
- [ ] Set up Vue Test Utils
- [ ] Write tests for shared components
  - [ ] TsiButton
  - [ ] TsiCard
  - [ ] TsiInput
  - [ ] TsiSpinner
  - [ ] TsiAlert
- [ ] Write tests for feature components

### E2E Tests
- [ ] Set up Playwright or Cypress
- [ ] Test critical user flows
  - [ ] File upload flow
  - [ ] Data visualization flows
  - [ ] Navigation flows

## 📚 Documentation Updates

- [ ] Add JSDoc comments to all public APIs
- [ ] Create component usage examples
- [ ] Document composable patterns
- [ ] Add contribution guidelines
- [ ] Create changelog

## 🔧 Development Experience

- [ ] Add pre-commit hooks (Husky)
- [ ] Configure Git hooks for linting
- [ ] Add commit message linting (commitlint)
- [ ] Set up continuous integration (CI)
- [ ] Configure build optimization

## 🚀 Performance Optimization

- [ ] Implement code splitting for routes
- [ ] Lazy load heavy components (charts)
- [ ] Optimize bundle size
- [ ] Add performance monitoring
- [ ] Optimize images and assets

## ♿ Accessibility

- [ ] Add ARIA labels to interactive elements
- [ ] Ensure keyboard navigation works
- [ ] Test with screen readers
- [ ] Add skip links
- [ ] Ensure sufficient color contrast

## 🌐 Internationalization (Optional)

- [ ] Set up Vue I18n
- [ ] Extract all text strings
- [ ] Create translation files
- [ ] Add language switcher

## 📦 Build & Deploy

- [ ] Optimize production build
- [ ] Configure environment variables
- [ ] Set up staging environment
- [ ] Document deployment process
- [ ] Add health check endpoint

## 📊 Monitoring & Analytics

- [ ] Add error tracking (Sentry)
- [ ] Add analytics (Google Analytics, Plausible)
- [ ] Monitor bundle size
- [ ] Track performance metrics

## 🔐 Security

- [ ] Review and sanitize user inputs
- [ ] Add CSRF protection
- [ ] Implement Content Security Policy
- [ ] Regular dependency updates
- [ ] Security audit

## 📋 Migration Steps Per Feature

Use this checklist for each feature:

1. [ ] Create feature folder structure
2. [ ] Move page component to feature folder
3. [ ] Rename to follow convention (`FeaturePage.vue`)
4. [ ] Convert to `<script setup>` syntax
5. [ ] Extract inline API calls to `apiService`
6. [ ] Create composable for feature logic
7. [ ] Move feature components to `components/`
8. [ ] Replace custom UI with shared components
9. [ ] Move styles to `styles/` folder
10. [ ] Apply design tokens to styles
11. [ ] Add TypeScript types
12. [ ] Update imports to use path aliases
13. [ ] Update router configuration
14. [ ] Test feature thoroughly
15. [ ] Update documentation

## 🎯 Success Criteria

- [ ] All features migrated to new structure
- [ ] No TypeScript errors
- [ ] All linting checks pass
- [ ] Code formatted consistently
- [ ] All routes working correctly
- [ ] All features visually consistent
- [ ] Performance not degraded
- [ ] Documentation complete and accurate

## 📝 Notes

- Migrate features incrementally (one at a time)
- Test thoroughly after each migration
- Keep old code until migration is verified
- Update documentation as you go
- Ask for code reviews

---

**Estimated Timeline**: 2-3 days per feature (8 features remaining)

**Priority Order**:
1. Sky Map (most complex, highest value)
2. Insights (second most complex)
3. Distributions (medium complexity)
4. Timeline, Trends, Compare, Visibility (simpler, can be parallelized)
