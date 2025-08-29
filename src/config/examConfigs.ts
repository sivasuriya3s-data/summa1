import { ExamConfig } from '../types';

export const examConfigs: Record<string, ExamConfig> = {
  neet: {
    name: 'NEET',
    formats: ['PDF', 'JPEG', 'JPG'],
    maxSizes: {
      PDF: 2 * 1024 * 1024, // 2MB
      JPEG: 500 * 1024, // 500KB
      JPG: 500 * 1024, // 500KB
    },
    requirements: [
      'Class 10th Marksheet',
      'Class 12th Marksheet', 
      'Category Certificate (if applicable)',
      'Passport Size Photo',
      'Signature'
    ]
  },
  jee: {
    name: 'JEE',
    formats: ['PDF', 'JPEG', 'PNG'],
    maxSizes: {
      PDF: 1 * 1024 * 1024, // 1MB
      JPEG: 300 * 1024, // 300KB
      PNG: 300 * 1024, // 300KB
    },
    requirements: [
      'Class 10th Certificate',
      'Class 12th Certificate',
      'Category Certificate',
      'Photo',
      'Signature'
    ]
  },
  upsc: {
    name: 'UPSC',
    formats: ['PDF', 'JPEG', 'JPG', 'PNG'],
    maxSizes: {
      PDF: 3 * 1024 * 1024, // 3MB
      JPEG: 1 * 1024 * 1024, // 1MB
      JPG: 1 * 1024 * 1024, // 1MB
      PNG: 1 * 1024 * 1024, // 1MB
    },
    requirements: [
      'Educational Certificates',
      'Age Proof',
      'Category Certificate',
      'Photo',
      'Signature',
      'Experience Certificates'
    ]
  },
  cat: {
    name: 'CAT',
    formats: ['PDF', 'JPEG'],
    maxSizes: {
      PDF: 1.5 * 1024 * 1024, // 1.5MB
      JPEG: 400 * 1024, // 400KB
    },
    requirements: [
      'Graduation Certificate',
      'Category Certificate',
      'Photo',
      'Signature'
    ]
  },
  gate: {
    name: 'GATE',
    formats: ['PDF', 'JPEG', 'PNG'],
    maxSizes: {
      PDF: 2 * 1024 * 1024, // 2MB
      JPEG: 500 * 1024, // 500KB
      PNG: 500 * 1024, // 500KB
    },
    requirements: [
      'Degree Certificate',
      'Category Certificate',
      'Photo',
      'Signature'
    ]
  }
};